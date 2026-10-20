use std::{collections::HashMap, fmt::Write as _, sync::Arc};

use super::model::Tu4Document;
use crate::diagram::{
    compile::{CompiledAction, CompiledDiagram, MachineId, StateId, compile},
    model::{document::DiagramDocument, node::Direction},
};

struct Instance {
    machine: MachineId,
    return_to: Option<u32>,
    nodes: Vec<u32>,
}

struct Converter {
    compiled: CompiledDiagram,
    symbols: Vec<char>,
    instances: Vec<Instance>,
    instance_ids: HashMap<(usize, Option<u32>), usize>,
    next_state: u32,
    source: String,
}

pub fn convert(document: &DiagramDocument) -> Result<Tu4Document, String> {
    let compiled = compile(document).map_err(|e| e.to_string())?;
    let entry = compiled.program().entry_machine();
    let mut converter = Converter {
        compiled,
        symbols: document.alphabet().iter().copied().collect(),
        instances: Vec::new(),
        instance_ids: HashMap::new(),
        next_state: 1,
        source: "// Converted from a diagram\n".into(),
    };
    let start = converter.instance(entry, None)?;
    for symbol in converter.symbols.clone() {
        converter.emit(0, symbol, symbol, start)?;
    }

    let mut cursor = 0;
    while cursor < converter.instances.len() {
        converter.expand(cursor)?;
        cursor += 1;
    }
    Ok(Tu4Document {
        alphabet: document.alphabet().clone(),
        source: Arc::from(converter.source),
    })
}

impl Converter {
    fn allocate(&mut self) -> Result<u32, String> {
        let state = self.next_state;
        self.next_state = state
            .checked_add(1)
            .ok_or("Converted program exceeds u32 state numbers")?;
        Ok(state)
    }

    fn instance(&mut self, machine: MachineId, return_to: Option<u32>) -> Result<u32, String> {
        if let Some(&index) = self.instance_ids.get(&(machine.0, return_to)) {
            let start = self
                .compiled
                .program()
                .machine(machine)
                .expect("compiled machine exists")
                .start();
            return Ok(*self
                .instances
                .get(index)
                .expect("instance was indexed")
                .nodes
                .get(start.0)
                .expect("start state exists"));
        }
        let compiled = self
            .compiled
            .program()
            .machine(machine)
            .expect("compiled machine exists");
        let start = compiled.start();
        let mut count = 0;
        while compiled.node(StateId(count)).is_some() {
            count += 1;
        }
        let mut nodes = Vec::new();
        nodes
            .try_reserve(count)
            .map_err(|error| format!("Cannot allocate converted states: {error}"))?;
        for _ in 0..count {
            nodes.push(self.allocate()?);
        }
        let entry = *nodes.get(start.0).expect("start state exists");
        self.instance_ids
            .insert((machine.0, return_to), self.instances.len());
        self.instances.push(Instance {
            machine,
            return_to,
            nodes,
        });
        Ok(entry)
    }

    fn transition(&self, instance: usize, state: StateId, read: char) -> u32 {
        let instance = self.instances.get(instance).expect("instance exists");
        let program = self.compiled.program();
        let symbol = program
            .symbol_id(read)
            .expect("symbol belongs to the compiled alphabet");
        let target = program
            .machine(instance.machine)
            .expect("compiled machine exists")
            .transition(state, symbol, program.alphabet_size())
            .expect("diagram transitions are complete");
        *instance
            .nodes
            .get(target.0)
            .expect("transition target exists")
    }

    fn emit(&mut self, state: u32, read: char, action: char, next: u32) -> Result<(), String> {
        self.source
            .try_reserve(48)
            .map_err(|error| format!("Cannot allocate converted source: {error}"))?;
        let read = if read == ' ' { 'λ' } else { read };
        let action = if action == ' ' { 'λ' } else { action };
        writeln!(self.source, "{state},{read},{action},{next}")
            .expect("writing to a String succeeds");
        Ok(())
    }

    fn move_rows(&mut self, state: u32, action: char, next: u32) -> Result<(), String> {
        for symbol in self.symbols.clone() {
            self.emit(state, symbol, action, next)?;
        }
        Ok(())
    }

    fn expand(&mut self, instance_id: usize) -> Result<(), String> {
        let instance = self.instances.get(instance_id).expect("instance exists");
        let machine_id = instance.machine;
        let nodes = instance.nodes.clone();
        let return_to = instance.return_to;
        let symbols = self.symbols.clone();
        for (index, &state) in nodes.iter().enumerate() {
            let node_id = StateId(index);
            let action = self
                .compiled
                .program()
                .machine(machine_id)
                .expect("compiled machine exists")
                .node(node_id)
                .expect("instance contains compiled nodes")
                .action()
                .clone();
            match action {
                CompiledAction::Move { direction, repeat } => {
                    self.next_state
                        .checked_add(repeat.get())
                        .ok_or("Converted program exceeds u32 state numbers")?;
                    let action = if direction == Direction::Left {
                        '<'
                    } else {
                        '>'
                    };
                    let mut current = state;
                    for _ in 0..repeat.get() {
                        let next = self.allocate()?;
                        self.move_rows(current, action, next)?;
                        current = next;
                    }
                    // A move chooses its outgoing edge using the symbol at the new head.
                    for &symbol in &symbols {
                        self.emit(
                            current,
                            symbol,
                            symbol,
                            self.transition(instance_id, node_id, symbol),
                        )?;
                    }
                }
                CompiledAction::Call { target, repeat } => {
                    let child = self
                        .compiled
                        .program()
                        .machine(target)
                        .expect("compiled call target exists");
                    let mut count = 0u64;
                    while child.node(StateId(count as usize)).is_some() {
                        count += 1;
                    }
                    if u64::from(self.next_state) + count * u64::from(repeat.get())
                        > u64::from(u32::MAX)
                    {
                        return Err("Converted program exceeds u32 state numbers".into());
                    }

                    let mut entries = HashMap::new();
                    for &symbol in &symbols {
                        let return_to = self.transition(instance_id, node_id, symbol);
                        let entry = if let Some(&entry) = entries.get(&return_to) {
                            entry
                        } else {
                            let mut entry = return_to;
                            for _ in 0..repeat.get() {
                                entry = self.instance(target, Some(entry))?;
                            }
                            entries.insert(return_to, entry);
                            entry
                        };
                        self.emit(state, symbol, symbol, entry)?;
                    }
                }
                CompiledAction::Start | CompiledAction::Write(_) | CompiledAction::Stop => {
                    for &symbol in &symbols {
                        let (write, next) = match action {
                            CompiledAction::Start => {
                                (symbol, self.transition(instance_id, node_id, symbol))
                            }
                            CompiledAction::Write(written) => {
                                (written, self.transition(instance_id, node_id, written))
                            }
                            CompiledAction::Stop => {
                                return_to.map_or(('~', 0), |next| (symbol, next))
                            }
                            _ => unreachable!("move and call were handled separately"),
                        };
                        self.emit(state, symbol, write, next)?;
                    }
                }
            }
        }
        Ok(())
    }
}
