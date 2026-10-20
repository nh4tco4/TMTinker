use std::num::NonZeroU32;

use slotmap::SecondaryMap;

use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::model::{
        document::{DiagramDocument, SubmachineDocument},
        node::{Direction, NodeAction},
        submachine::{NodeKey, SubmachineKey},
        transition::EdgeKey,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MachineId(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateId(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolId(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub submachine: SubmachineKey,
    pub node: NodeKey,
}

#[derive(Clone, Debug)]
pub enum CompiledAction {
    Start,
    Stop,
    Move {
        direction: Direction,
        repeat: NonZeroU32,
    },
    Write(char),
    Call {
        target: MachineId,
        repeat: NonZeroU32,
    },
}

#[derive(Clone)]
pub struct CompiledDiagram {
    source: DiagramDocument,
    program: CompiledProgram,
}

#[derive(Clone)]
pub struct CompiledProgram {
    alphabet: CompiledAlphabet,
    entry_machine: MachineId,
    machines: Vec<CompiledMachine>,
}

#[derive(Clone)]
struct CompiledAlphabet {
    symbols: Vec<char>,
}

#[derive(Clone)]
pub struct CompiledMachine {
    source: SubmachineKey,
    start: StateId,
    nodes: Vec<CompiledNode>,
    transitions: Vec<Option<StateId>>,
}

#[derive(Clone)]
pub struct CompiledNode {
    source: NodeKey,
    action: CompiledAction,
}

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    #[error("{0}")]
    InvalidCallGraph(String),
    #[error("The diagram has no entry submachine.")]
    MissingEntrySubmachine,
    #[error("Submachine {machine:?} must have exactly one Start node (found {count}).")]
    InvalidStartCount {
        machine: SubmachineKey,
        count: usize,
    },
    #[error("Start node {node:?} in submachine {machine:?} cannot have incoming transitions.")]
    StartHasIncoming {
        machine: SubmachineKey,
        node: NodeKey,
    },
    #[error("Stop node {node:?} in submachine {machine:?} cannot have outgoing transitions.")]
    StopHasOutgoing {
        machine: SubmachineKey,
        node: NodeKey,
    },
    #[error("Node {node:?} in submachine {machine:?} calls a missing submachine {target:?}.")]
    MissingSubmachine {
        machine: SubmachineKey,
        node: NodeKey,
        target: SubmachineKey,
    },
    #[error("An edge in submachine {machine:?} references missing node {node:?}.")]
    MissingNode {
        machine: SubmachineKey,
        node: NodeKey,
    },
    #[error(
        "Transition from node {node:?} in submachine {machine:?} uses unknown symbol {symbol:?}."
    )]
    UnknownSymbol {
        machine: SubmachineKey,
        node: NodeKey,
        symbol: char,
    },
    #[error("Node {node:?} in submachine {machine:?} has conflicting edges {first:?} and {second:?} for {display:?}.", display = if *symbol == ' ' { 'λ' } else { *symbol })]
    AmbiguousTransition {
        machine: SubmachineKey,
        node: NodeKey,
        symbol: char,
        first: EdgeKey,
        second: EdgeKey,
    },
    #[error("Node {node:?} in submachine {machine:?} has no transition for {display:?}.", display = if *symbol == ' ' { 'λ' } else { *symbol })]
    MissingTransition {
        machine: SubmachineKey,
        node: NodeKey,
        symbol: char,
    },
}

impl CompiledDiagram {
    pub fn source(&self) -> &DiagramDocument {
        &self.source
    }

    pub fn program(&self) -> &CompiledProgram {
        &self.program
    }
}

impl CompiledProgram {
    pub fn entry_machine(&self) -> MachineId {
        self.entry_machine
    }

    pub fn machine(&self, id: MachineId) -> Option<&CompiledMachine> {
        self.machines.get(id.0)
    }

    pub fn symbol_id(&self, symbol: char) -> Option<SymbolId> {
        self.alphabet.symbol_id(symbol)
    }

    pub fn alphabet_size(&self) -> usize {
        self.alphabet.len()
    }

    pub fn source_location(&self, machine: MachineId, state: StateId) -> Option<SourceLocation> {
        let machine = self.machine(machine)?;
        let node = machine.node(state)?;
        Some(SourceLocation {
            submachine: machine.source,
            node: node.source,
        })
    }
}

impl CompiledAlphabet {
    fn new(source: &ValidatedAlphabet) -> Self {
        Self {
            symbols: source.iter().copied().collect(),
        }
    }

    fn len(&self) -> usize {
        self.symbols.len()
    }

    fn symbol_id(&self, symbol: char) -> Option<SymbolId> {
        self.symbols
            .iter()
            .position(|item| *item == symbol)
            .map(SymbolId)
    }
}

impl CompiledMachine {
    pub fn start(&self) -> StateId {
        self.start
    }

    pub fn node(&self, state: StateId) -> Option<&CompiledNode> {
        self.nodes.get(state.0)
    }

    pub fn transition(
        &self,
        state: StateId,
        symbol: SymbolId,
        alphabet_size: usize,
    ) -> Option<StateId> {
        self.transitions
            .get(state.0.checked_mul(alphabet_size)?.checked_add(symbol.0)?)
            .copied()
            .flatten()
    }
}

impl CompiledNode {
    pub fn action(&self) -> &CompiledAction {
        &self.action
    }
}

pub fn compile(source: &DiagramDocument) -> Result<CompiledDiagram, CompileError> {
    source
        .validate_calls()
        .map_err(CompileError::InvalidCallGraph)?;
    let program = compile_program(source)?;
    Ok(CompiledDiagram {
        source: source.clone(),
        program,
    })
}

fn compile_program(source: &DiagramDocument) -> Result<CompiledProgram, CompileError> {
    let mut machine_ids = SecondaryMap::<SubmachineKey, MachineId>::new();
    for (index, (key, _)) in source.submachines().enumerate() {
        machine_ids.insert(key, MachineId(index));
    }

    let entry_machine = machine_ids
        .get(source.entry_submachine())
        .copied()
        .ok_or(CompileError::MissingEntrySubmachine)?;
    let alphabet = CompiledAlphabet::new(source.alphabet());
    let machines = source
        .submachines()
        .map(|(key, document)| compile_machine(key, document, &machine_ids, &alphabet))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CompiledProgram {
        alphabet,
        entry_machine,
        machines,
    })
}

fn compile_machine(
    source_key: SubmachineKey,
    document: &SubmachineDocument,
    machine_ids: &SecondaryMap<SubmachineKey, MachineId>,
    alphabet: &CompiledAlphabet,
) -> Result<CompiledMachine, CompileError> {
    let machine = document.machine();
    let mut state_ids = SecondaryMap::<NodeKey, StateId>::new();
    for (index, (key, _)) in machine.nodes_iter().enumerate() {
        state_ids.insert(key, StateId(index));
    }

    let starts: Vec<_> = machine
        .nodes_iter()
        .filter(|(_, node)| matches!(node.action(), NodeAction::Start))
        .map(|(key, _)| key)
        .collect();
    let start_node = match starts.as_slice() {
        [start] => *start,
        _ => {
            return Err(CompileError::InvalidStartCount {
                machine: source_key,
                count: starts.len(),
            });
        }
    };

    if machine.incoming_edges(start_node).next().is_some() {
        return Err(CompileError::StartHasIncoming {
            machine: source_key,
            node: start_node,
        });
    }

    let nodes = machine
        .nodes_iter()
        .map(|(source, node)| {
            Ok(CompiledNode {
                source,
                action: compile_action(source_key, source, node.action(), machine_ids)?,
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    let start = *state_ids.get(start_node).ok_or(CompileError::MissingNode {
        machine: source_key,
        node: start_node,
    })?;

    let mut transitions = vec![None; nodes.len() * alphabet.len()];
    let mut transition_edges = vec![None; transitions.len()];
    for (edge_key, edge) in machine.edges_iter() {
        let source_state = *state_ids
            .get(edge.source())
            .ok_or(CompileError::MissingNode {
                machine: source_key,
                node: edge.source(),
            })?;

        let target_state = *state_ids
            .get(edge.target())
            .ok_or(CompileError::MissingNode {
                machine: source_key,
                node: edge.target(),
            })?;

        for symbol in edge.symbols() {
            alphabet
                .symbol_id(symbol)
                .ok_or(CompileError::UnknownSymbol {
                    machine: source_key,
                    node: edge.source(),
                    symbol,
                })?;
        }
        for (symbol_index, symbol) in alphabet.symbols.iter().copied().enumerate() {
            if edge.symbols().contains(symbol) == edge.inverse() {
                continue;
            }
            let symbol_id = SymbolId(symbol_index);
            let index = source_state.0 * alphabet.len() + symbol_id.0;
            let transition = transitions
                .get_mut(index)
                .expect("State and symbol IDs were allocated for this transition table.");
            if let Some(first) = transition_edges
                .get_mut(index)
                .expect("State and symbol IDs were allocated for this transition table.")
                .replace(edge_key)
            {
                return Err(CompileError::AmbiguousTransition {
                    machine: source_key,
                    node: edge.source(),
                    symbol,
                    first,
                    second: edge_key,
                });
            }
            *transition = Some(target_state);
        }
    }

    for (state_index, node) in nodes.iter().enumerate() {
        if matches!(node.action, CompiledAction::Stop) {
            if machine.outgoing_edges(node.source).next().is_some() {
                return Err(CompileError::StopHasOutgoing {
                    machine: source_key,
                    node: node.source,
                });
            }
            continue;
        }
        for (symbol_index, symbol) in alphabet.symbols.iter().copied().enumerate() {
            let index = state_index * alphabet.len() + symbol_index;
            if transitions
                .get(index)
                .expect(
                    "Unreachable. State and symbol IDs were allocated for this transition table.",
                )
                .is_none()
            {
                return Err(CompileError::MissingTransition {
                    machine: source_key,
                    node: node.source,
                    symbol,
                });
            }
        }
    }

    Ok(CompiledMachine {
        source: source_key,
        start,
        nodes,
        transitions,
    })
}

fn compile_action(
    machine: SubmachineKey,
    node: NodeKey,
    action: &NodeAction,
    machine_ids: &SecondaryMap<SubmachineKey, MachineId>,
) -> Result<CompiledAction, CompileError> {
    match action {
        NodeAction::Start => Ok(CompiledAction::Start),
        NodeAction::Stop => Ok(CompiledAction::Stop),
        NodeAction::Move { direction, repeat } => Ok(CompiledAction::Move {
            direction: direction.clone(),
            repeat: *repeat,
        }),
        NodeAction::Write(symbol) => Ok(CompiledAction::Write(*symbol)),
        NodeAction::Call { target, repeat } => {
            let target_id =
                machine_ids
                    .get(*target)
                    .copied()
                    .ok_or(CompileError::MissingSubmachine {
                        machine,
                        node,
                        target: *target,
                    })?;
            Ok(CompiledAction::Call {
                target: target_id,
                repeat: *repeat,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::tape::Tape,
        diagram::{
            compile::diagnostics::{NodeProblem, node_problems},
            model::{layout::LayoutPos, transition::TransitionSymbols},
            runtime::{DiagramRunSession, RunStatus},
        },
        persistence,
        project::{Project, model::ProjectName},
    };

    fn fixture() -> (DiagramDocument, NodeKey, NodeKey, EdgeKey, EdgeKey) {
        let alphabet = ValidatedAlphabet::new(" 0123456789".chars()).expect("valid test fixture");
        let mut document = DiagramDocument::new(alphabet.clone());
        let machine = document
            .submachine_mut(document.entry_submachine())
            .expect("valid test fixture");
        let start = machine.add_node(LayoutPos::new(0., 0.), NodeAction::Start);
        let stop = machine.add_node(LayoutPos::new(150., 0.), NodeAction::Stop);
        let digits = machine
            .add_edge(
                TransitionSymbols::new([' '], &alphabet).expect("valid test fixture"),
                start,
                stop,
            )
            .expect("valid test fixture");
        machine
            .machine_mut()
            .get_edge_mut(digits)
            .expect("valid test fixture")
            .set_inverse(true);
        let blank = machine
            .add_edge(
                TransitionSymbols::new([' '], &alphabet).expect("valid test fixture"),
                start,
                stop,
            )
            .expect("valid test fixture");
        (document, start, stop, digits, blank)
    }

    #[test]
    fn inverse_blank_runs_for_each_digit_and_agrees_with_editor_diagnostics() {
        let (document, start, stop, digits, blank) = fixture();
        let machine = document
            .submachine(document.entry_submachine())
            .expect("valid test fixture")
            .machine();
        assert!(node_problems(machine, document.alphabet(), start).is_empty());
        let compiled = compile(&document).expect("valid test fixture");
        for symbol in document.alphabet().iter().copied() {
            let mut tape = Tape::new();
            tape.set_cell(0, symbol);
            let mut session = DiagramRunSession::new(compiled.clone(), tape.clone());
            assert_eq!(
                session.next_transition(),
                Some(if symbol == ' ' { blank } else { digits })
            );
            session.step();
            assert_eq!(session.source_location().node, stop);
            session.step();
            assert_eq!(session.status(), RunStatus::Halted);
            assert_eq!(session.tape(), &tape);
        }
    }

    #[test]
    fn inverse_overlap_is_an_error_even_with_the_same_target() {
        let (mut document, start, _, digits, blank) = fixture();
        let alphabet = document.alphabet().clone();
        let machine = document
            .submachine_mut(document.entry_submachine())
            .expect("valid test fixture")
            .machine_mut();
        machine
            .get_edge_mut(blank)
            .expect("valid test fixture")
            .set_symbols(TransitionSymbols::new(['5'], &alphabet).expect("valid test fixture"));
        assert!(
            node_problems(machine, &alphabet, start)
                .contains(&NodeProblem::AmbiguousTransition('5'))
        );
        assert!(
            matches!(compile(&document), Err(CompileError::AmbiguousTransition {
            symbol: '5', first, second, ..
        }) if first == digits && second == blank)
        );
        // Two inverse edges overlap as well.
        document
            .submachine_mut(document.entry_submachine())
            .expect("valid test fixture")
            .machine_mut()
            .get_edge_mut(blank)
            .expect("valid test fixture")
            .set_inverse(true);
        assert!(matches!(
            compile(&document),
            Err(CompileError::AmbiguousTransition { .. })
        ));
    }

    #[test]
    fn inverse_does_not_cover_an_excluded_blank() {
        let (mut document, start, _, _, blank) = fixture();
        document
            .submachine_mut(document.entry_submachine())
            .expect("valid test fixture")
            .remove_edge(blank);
        assert!(
            matches!(compile(&document), Err(CompileError::MissingTransition { node, symbol: ' ', .. }) if node == start)
        );
    }

    #[test]
    fn inverse_extremes_and_project_roundtrip() {
        for exclude_all in [false, true] {
            let (mut document, _, _, digits, blank) = fixture();
            let alphabet = document.alphabet().clone();
            let machine = document
                .submachine_mut(document.entry_submachine())
                .expect("valid test fixture")
                .machine_mut();
            let edge = machine.get_edge_mut(digits).expect("valid test fixture");
            edge.set_symbols(
                TransitionSymbols::new_exclusions(
                    alphabet.iter().copied().filter(|_| exclude_all),
                    &alphabet,
                )
                .expect("valid test fixture"),
            );
            assert!(!edge.accepts('a', &alphabet));
            if exclude_all {
                machine
                    .get_edge_mut(blank)
                    .expect("valid test fixture")
                    .set_symbols(
                        TransitionSymbols::new(alphabet.iter().copied(), &alphabet)
                            .expect("valid test fixture"),
                    );
            } else {
                machine.remove_edge(blank);
            }
            document.validate().expect("valid test fixture");
            assert!(compile(&document).is_ok());
            let project = Project::new(
                ProjectName::new("Inverse".into()).expect("valid test fixture"),
                document,
            );
            let loaded = persistence::decode_project(
                &persistence::encode_project(&project).expect("valid test fixture"),
            )
            .expect("valid test fixture");
            assert!(loaded.document() == project.document());
            assert!(compile(loaded.document()).is_ok());
            let library = persistence::decode_library(
                &persistence::encode_library([&project].into_iter()).expect("valid test fixture"),
            )
            .expect("valid test fixture");
            assert!(library[0].document() == project.document());
        }
    }

    #[test]
    fn old_edges_without_inverse_remain_inclusions() {
        let (document, _, _, _, blank) = fixture();
        let edge = document
            .submachine(document.entry_submachine())
            .expect("valid test fixture")
            .machine()
            .edges_iter()
            .find(|(key, _)| *key == blank)
            .expect("valid test fixture")
            .1;
        let mut json = serde_json::to_value(edge).expect("valid test fixture");
        json.as_object_mut()
            .expect("valid test fixture")
            .remove("inverse");
        let loaded: crate::diagram::model::transition::Edge =
            serde_json::from_value(json).expect("valid test fixture");
        assert!(!loaded.inverse());
        assert!(loaded.accepts(' ', document.alphabet()));
        assert!(!loaded.accepts('5', document.alphabet()));
    }
}
