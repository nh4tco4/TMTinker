use crate::{
    core::tape::Tape,
    diagram::{
        compile::{CompiledAction, CompiledDiagram, MachineId, SourceLocation, StateId},
        model::{node::Direction, transition::EdgeKey},
        runtime::RunStatus,
    },
    shared::runtime::RunSession,
};

pub struct DiagramRunSession {
    compiled: CompiledDiagram,
    initial_tape: Tape,
    tape: Tape,
    location: RuntimeLocation,
    stack: Vec<CallFrame>,
    status: RunStatus,
    fault: Option<String>,
    steps: u64,
}

#[derive(Clone, Copy)]
struct RuntimeLocation {
    machine: MachineId,
    state: StateId,
}

struct CallFrame {
    target_machine: MachineId,
    return_location: RuntimeLocation,
    remaining: u32,
}

impl DiagramRunSession {
    pub fn new(compiled: CompiledDiagram, tape: Tape) -> Self {
        let entry_machine = compiled.program().entry_machine();
        let start = compiled
            .program()
            .machine(entry_machine)
            .expect("compiled entry machine exists")
            .start();
        Self {
            compiled,
            initial_tape: tape.clone(),
            tape,
            location: RuntimeLocation {
                machine: entry_machine,
                state: start,
            },
            stack: Vec::new(),
            status: RunStatus::Paused,
            fault: None,
            steps: 0,
        }
    }

    pub fn compiled(&self) -> &CompiledDiagram {
        &self.compiled
    }

    pub fn tape(&self) -> &Tape {
        &self.tape
    }

    pub fn replace_tape(&mut self, tape: Tape) {
        self.initial_tape = tape;
        self.reset();
    }

    pub fn status(&self) -> RunStatus {
        self.status
    }

    pub fn fault(&self) -> Option<&str> {
        self.fault.as_deref()
    }

    pub fn steps(&self) -> u64 {
        self.steps
    }

    pub fn source_location(&self) -> SourceLocation {
        self.compiled
            .program()
            .source_location(self.location.machine, self.location.state)
            .expect("runtime location belongs to compiled program")
    }

    pub fn next_transition(&self) -> Option<EdgeKey> {
        if matches!(self.status, RunStatus::Halted | RunStatus::Faulted) {
            return None;
        }
        let action = self
            .compiled
            .program()
            .machine(self.location.machine)?
            .node(self.location.state)?
            .action();
        let symbol = match action {
            CompiledAction::Stop => return None,
            CompiledAction::Write(symbol) => *symbol,
            CompiledAction::Move { direction, repeat } => self.tape.cell(moved_head_position(
                self.tape.head_position(),
                direction,
                *repeat,
            )),
            CompiledAction::Start | CompiledAction::Call { .. } => self.tape.read(),
        };
        let location = self.source_location();
        self.compiled
            .source()
            .submachine(location.submachine)?
            .machine()
            .edges_iter()
            .find(|(_, edge)| {
                edge.source() == location.node
                    && edge.accepts(symbol, self.compiled.source().alphabet())
            })
            .map(|(key, _)| key)
    }

    pub fn run(&mut self) {
        if matches!(self.status, RunStatus::Paused) {
            self.status = RunStatus::Running;
        }
    }

    pub fn pause(&mut self) {
        if matches!(self.status, RunStatus::Running) {
            self.status = RunStatus::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.status = RunStatus::Halted;
    }

    pub fn reset(&mut self) {
        let entry_machine = self.compiled.program().entry_machine();
        let start = self
            .compiled
            .program()
            .machine(entry_machine)
            .expect("compiled entry machine exists")
            .start();
        self.tape = self.initial_tape.clone();
        self.location = RuntimeLocation {
            machine: entry_machine,
            state: start,
        };
        self.stack.clear();
        self.status = RunStatus::Paused;
        self.fault = None;
        self.steps = 0;
    }

    pub fn step(&mut self) {
        if matches!(self.status, RunStatus::Halted | RunStatus::Faulted) {
            return;
        }
        if let Err(message) = self.try_step() {
            self.status = RunStatus::Faulted;
            self.fault = Some(message);
        }
    }

    fn try_step(&mut self) -> Result<(), String> {
        let action = self
            .compiled
            .program()
            .machine(self.location.machine)
            .and_then(|machine| machine.node(self.location.state))
            .map(|node| node.action().clone())
            .ok_or_else(|| "Runtime state is missing from the compiled program".to_owned())?;

        match action {
            CompiledAction::Start => self.advance()?,
            CompiledAction::Stop => self.return_or_halt()?,
            CompiledAction::Move { direction, repeat } => {
                if repeat.get() == 1 {
                    match direction {
                        Direction::Left => self.tape.move_left(),
                        Direction::Right => self.tape.move_right(),
                    }
                } else {
                    self.tape.set_head(moved_head_position(
                        self.tape.head_position(),
                        &direction,
                        repeat,
                    ));
                }
                self.advance()?;
            }
            CompiledAction::Write(symbol) => {
                self.tape.write(symbol);
                self.advance()?;
            }
            CompiledAction::Call { target, repeat } => {
                let return_location = RuntimeLocation {
                    machine: self.location.machine,
                    state: self.next_state()?,
                };
                let start = self
                    .compiled
                    .program()
                    .machine(target)
                    .ok_or_else(|| "Called machine is missing".to_owned())?
                    .start();
                self.stack.push(CallFrame {
                    target_machine: target,
                    return_location,
                    remaining: repeat.get(),
                });
                self.location = RuntimeLocation {
                    machine: target,
                    state: start,
                };
            }
        }

        self.steps = self.steps.saturating_add(1);
        Ok(())
    }

    fn advance(&mut self) -> Result<(), String> {
        self.location.state = self.next_state()?;
        Ok(())
    }

    fn next_state(&self) -> Result<StateId, String> {
        let program = self.compiled.program();
        let symbol = self.tape.read();
        let symbol_id = program
            .symbol_id(symbol)
            .ok_or_else(|| format!("Tape contains symbol {symbol:?} outside the alphabet"))?;
        program
            .machine(self.location.machine)
            .and_then(|machine| {
                machine.transition(self.location.state, symbol_id, program.alphabet_size())
            })
            .ok_or_else(|| "Compiled transition is missing".to_owned())
    }

    fn return_or_halt(&mut self) -> Result<(), String> {
        let Some(mut frame) = self.stack.pop() else {
            self.status = RunStatus::Halted;
            return Ok(());
        };

        if frame.remaining > 1 {
            frame.remaining -= 1;
            let start = self
                .compiled
                .program()
                .machine(frame.target_machine)
                .ok_or_else(|| "Called machine is missing".to_owned())?
                .start();
            self.location = RuntimeLocation {
                machine: frame.target_machine,
                state: start,
            };
            self.stack.push(frame);
        } else {
            self.location = frame.return_location;
        }
        Ok(())
    }
}

fn moved_head_position(head: i32, direction: &Direction, repeat: std::num::NonZeroU32) -> i32 {
    let delta = match direction {
        Direction::Left => -i64::from(repeat.get()),
        Direction::Right => i64::from(repeat.get()),
    };
    (i64::from(head) + delta).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

impl RunSession for crate::diagram::runtime::DiagramRunSession {
    type Program = crate::diagram::compile::CompiledDiagram;

    fn new(program: Self::Program, tape: Tape) -> Self {
        Self::new(program, tape)
    }
    fn tape(&self) -> &Tape {
        self.tape()
    }
    fn status(&self) -> RunStatus {
        self.status()
    }
    fn steps(&self) -> u64 {
        self.steps()
    }
    fn fault(&self) -> Option<&str> {
        self.fault()
    }
    fn step(&mut self) {
        self.step();
    }
    fn run(&mut self) {
        self.run();
    }
    fn pause(&mut self) {
        self.pause();
    }
    fn stop(&mut self) {
        self.stop();
    }
    fn reset(&mut self) {
        self.reset();
    }
    fn program(&self) -> Self::Program {
        self.compiled().clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{alphabet::ValidatedAlphabet, tape::BLANK},
        diagram::{
            compile::compile,
            model::{
                document::DiagramDocument, layout::LayoutPos, node::NodeAction,
                transition::TransitionSymbols,
            },
            runtime::{DiagramRunSession, RunStatus},
        },
    };

    use super::Tape;

    #[test]
    fn compiled_session_keeps_source_mapping_and_executes() {
        let alphabet = ValidatedAlphabet::new([BLANK, '1']).expect("valid test alphabet");
        let mut document = DiagramDocument::new(alphabet.clone());
        let entry = document.entry_submachine();
        let machine = document
            .submachine_mut(entry)
            .expect("entry submachine exists");
        let start = machine.add_node(LayoutPos::new(0.0, 0.0), NodeAction::Start);
        let write = machine.add_node(LayoutPos::new(75.0, 0.0), NodeAction::Write('1'));
        let stop = machine.add_node(LayoutPos::new(150.0, 0.0), NodeAction::Stop);
        let every_symbol =
            || TransitionSymbols::new([BLANK, '1'], &alphabet).expect("valid transition symbols");
        machine
            .add_edge(every_symbol(), start, write)
            .expect("test nodes exist");
        machine
            .add_edge(every_symbol(), write, stop)
            .expect("test nodes exist");

        let compiled = compile(&document).expect("test diagram compiles");
        let mut session = DiagramRunSession::new(compiled, Tape::new());
        assert_eq!(session.source_location().node, start);

        session.step();
        assert_eq!(session.source_location().node, write);
        session.step();
        assert_eq!(session.tape().read(), '1');
        assert_eq!(session.source_location().node, stop);
        session.step();
        assert_eq!(session.status(), RunStatus::Halted);
    }
    #[test]
    fn transition_preview_accounts_for_writes_moves_and_run_status_without_mutating_tape() {
        use crate::diagram::model::node::Direction;
        use std::num::NonZeroU32;
        for direction in [Direction::Left, Direction::Right] {
            for distance in [1, 3] {
                let alphabet = ValidatedAlphabet::new([BLANK, '1']).unwrap();
                let mut document = DiagramDocument::new(alphabet.clone());
                let entry = document.entry_submachine();
                let machine = document.submachine_mut(entry).unwrap();
                let start = machine.add_node(LayoutPos::new(0., 0.), NodeAction::Start);
                let write = machine.add_node(LayoutPos::new(75., 0.), NodeAction::Write('1'));
                let movement = machine.add_node(
                    LayoutPos::new(150., 0.),
                    NodeAction::Move {
                        direction: direction.clone(),
                        repeat: NonZeroU32::new(distance).unwrap(),
                    },
                );
                let stop = machine.add_node(LayoutPos::new(225., 0.), NodeAction::Stop);
                let symbols = |chars: Vec<char>| TransitionSymbols::new(chars, &alphabet).unwrap();
                let initial = machine
                    .add_edge(symbols(vec![BLANK, '1']), start, write)
                    .unwrap();
                machine.add_edge(symbols(vec![BLANK]), write, stop).unwrap();
                let after_write = machine
                    .add_edge(symbols(vec!['1']), write, movement)
                    .unwrap();
                let after_move = machine
                    .add_edge(symbols(vec![BLANK]), movement, stop)
                    .unwrap();
                machine
                    .add_edge(symbols(vec!['1']), movement, write)
                    .unwrap();
                let mut session = DiagramRunSession::new(compile(&document).unwrap(), Tape::new());
                assert_eq!(session.next_transition(), Some(initial));
                session.step();
                assert_eq!(session.tape().read(), BLANK);
                let before = session.tape().clone();
                assert_eq!(session.next_transition(), Some(after_write));
                assert_eq!(session.next_transition(), Some(after_write));
                assert_eq!(session.tape(), &before);
                assert_eq!(session.steps(), 1);
                session.step();
                assert_eq!(session.source_location().node, movement);
                assert_eq!(session.tape().read(), '1');
                assert_eq!(session.next_transition(), Some(after_move));
                session.run();
                assert_eq!(session.next_transition(), Some(after_move));
                session.step();
                assert_eq!(session.source_location().node, stop);
                assert_eq!(session.next_transition(), None);
                session.step();
                assert_eq!(session.next_transition(), None);
                session.reset();
                assert_eq!(session.next_transition(), Some(initial));
                session.stop();
                assert_eq!(session.next_transition(), None);
                let mut invalid = Tape::new();
                invalid.set_cell(0, '?');
                session.replace_tape(invalid);
                assert_eq!(session.next_transition(), None);
                session.step();
                assert_eq!(session.status(), RunStatus::Faulted);
                assert_eq!(session.next_transition(), None);
            }
        }
    }
}
