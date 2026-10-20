use slotmap::SlotMap;

use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::model::layout::{GraphLayout, LayoutPos},
    diagram::model::{
        node::NodeAction,
        submachine::{NodeKey, Submachine, SubmachineKey, SubmachineName},
        transition::{EdgeKey, TransitionSymbols},
    },
};

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SubmachineDocument {
    machine: Submachine,
    layout: GraphLayout,
    #[serde(default)]
    system: bool,
}

impl SubmachineDocument {
    pub fn new(name: SubmachineName) -> Self {
        Self {
            machine: Submachine::new(name),
            layout: GraphLayout::new(),
            system: false,
        }
    }

    pub fn machine(&self) -> &Submachine {
        &self.machine
    }

    pub fn is_system(&self) -> bool {
        self.system
    }

    pub fn machine_mut(&mut self) -> &mut Submachine {
        &mut self.machine
    }

    pub fn layout(&self) -> &GraphLayout {
        &self.layout
    }

    pub fn layout_mut(&mut self) -> &mut GraphLayout {
        &mut self.layout
    }

    pub fn add_node(&mut self, position: LayoutPos, action: NodeAction) -> NodeKey {
        let key = self.machine.add_node(action);

        self.layout.set_node_position(key, position);

        key
    }

    pub fn remove_node(&mut self, key: NodeKey) {
        self.machine.remove_node(key);
        self.layout.remove_node(key);
    }

    pub fn add_edge(
        &mut self,
        symbols: TransitionSymbols,
        source: NodeKey,
        target: NodeKey,
    ) -> Option<EdgeKey> {
        if self.machine.get_node(source).is_none() || self.machine.get_node(target).is_none() {
            return None;
        }
        Some(self.machine.add_edge(symbols, source, target))
    }

    pub fn remove_edge(&mut self, key: EdgeKey) {
        self.machine.remove_edge(key);
    }

    pub fn move_node_unsnapped(&mut self, key: NodeKey, delta: LayoutPos) {
        self.layout.update_node_position_unsnapped(key, delta);
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagramDocument {
    alphabet: ValidatedAlphabet,
    submachines: SlotMap<SubmachineKey, SubmachineDocument>,
    entry_submachine: SubmachineKey,
}

impl PartialEq for DiagramDocument {
    fn eq(&self, other: &Self) -> bool {
        self.alphabet == other.alphabet
            && self.entry_submachine == other.entry_submachine
            && self.submachines.iter().eq(other.submachines.iter())
    }
}

impl DiagramDocument {
    pub fn new(alphabet: ValidatedAlphabet) -> Self {
        let mut submachines = SlotMap::with_key();

        let entry_submachine = submachines.insert(SubmachineDocument::new(
            SubmachineName::parse("Main machine".to_owned()).expect("static name is valid"),
        ));

        Self {
            alphabet,
            submachines,
            entry_submachine,
        }
    }

    pub fn submachine(&self, key: SubmachineKey) -> Option<&SubmachineDocument> {
        self.submachines.get(key)
    }

    pub fn submachine_mut(&mut self, key: SubmachineKey) -> Option<&mut SubmachineDocument> {
        self.submachines.get_mut(key)
    }

    pub fn submachines(&self) -> impl Iterator<Item = (SubmachineKey, &SubmachineDocument)> {
        self.submachines.iter()
    }

    pub fn entry_submachine(&self) -> SubmachineKey {
        self.entry_submachine
    }

    pub fn alphabet(&self) -> &ValidatedAlphabet {
        &self.alphabet
    }

    pub fn add_submachine(&mut self, name: SubmachineName) -> SubmachineKey {
        self.submachines.insert(SubmachineDocument::new(name))
    }

    pub fn can_remove_submachine(&self, key: SubmachineKey) -> Result<(), String> {
        let machine = self.submachine(key).ok_or("Submachine no longer exists")?;
        if machine.is_system() {
            return Err("System machines are read-only".into());
        }
        if key == self.entry_submachine {
            return Err("The main machine cannot be deleted".into());
        }
        for (caller_key, caller) in self.submachines() {
            if caller_key != key && caller.machine().nodes_iter().any(|(_, node)| {
                matches!(node.action(), NodeAction::Call { target, .. } if *target == key)
            }) {
                return Err(format!("Remove calls from '{}' before deleting this submachine", caller.machine().name()));
            }
        }
        Ok(())
    }

    pub fn remove_submachine(&mut self, key: SubmachineKey) -> Result<(), String> {
        self.can_remove_submachine(key)?;
        self.submachines.remove(key);
        Ok(())
    }

    pub fn replace_submachine(
        &mut self,
        key: SubmachineKey,
        value: SubmachineDocument,
    ) -> Result<(), String> {
        let current = self
            .submachines
            .get_mut(key)
            .ok_or("Submachine no longer exists")?;
        if current.is_system() {
            return Err("System machines are read-only".into());
        }
        if value.is_system() {
            return Err("User commands cannot create system machines".into());
        }
        *current = value;
        Ok(())
    }

    pub fn validate_calls(&self) -> Result<(), String> {
        use std::collections::{HashMap, VecDeque};

        let mut incoming: HashMap<_, usize> = self.submachines().map(|(key, _)| (key, 0)).collect();
        let mut calls: HashMap<_, Vec<_>> = HashMap::new();
        for (key, document) in self.submachines() {
            for (_, node) in document.machine().nodes_iter() {
                if let NodeAction::Call { target, .. } = node.action() {
                    *incoming
                        .get_mut(target)
                        .ok_or("Missing called submachine")? += 1;
                    calls.entry(key).or_default().push(*target);
                }
            }
        }
        let mut ready: VecDeque<_> = incoming
            .iter()
            .filter_map(|(&key, &count)| (count == 0).then_some(key))
            .collect();
        let mut visited = 0;
        while let Some(key) = ready.pop_front() {
            visited += 1;
            if let Some(targets) = calls.get(&key) {
                for target in targets {
                    let count = incoming.get_mut(target).expect("call target was checked");
                    *count -= 1;
                    if *count == 0 {
                        ready.push_back(*target);
                    }
                }
            }
        }
        if visited != incoming.len() {
            return Err("Recursive submachine calls are not allowed".into());
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        self.validate_calls()?;
        ValidatedAlphabet::new(self.alphabet.iter().copied()).map_err(|e| e.to_string())?;
        if self.submachine(self.entry_submachine).is_none() {
            return Err("Missing main machine".into());
        }
        for (_, document) in self.submachines() {
            SubmachineName::parse(document.machine().name().to_owned())
                .map_err(|e| e.to_string())?;
            for (key, node) in document.machine().nodes_iter() {
                let position = document
                    .layout()
                    .get_node_position(key)
                    .ok_or("Missing node position")?;
                if !position.x.is_finite() || !position.y.is_finite() {
                    return Err("Invalid node position".into());
                }
                match node.action() {
                    NodeAction::Call { target, .. } if self.submachine(*target).is_none() => {
                        return Err("Missing called machine".into());
                    }
                    NodeAction::Write(c) if !self.alphabet.contains(*c) => {
                        return Err("Invalid written symbol".into());
                    }
                    _ => {}
                }
            }
            for (_, edge) in document.machine().edges_iter() {
                if document.machine().get_node(edge.source()).is_none()
                    || document.machine().get_node(edge.target()).is_none()
                {
                    return Err("Transition references a missing node".into());
                }
                if edge.inverse() {
                    TransitionSymbols::new_exclusions(edge.symbols().iter(), &self.alphabet)
                } else {
                    TransitionSymbols::new(edge.symbols().iter(), &self.alphabet)
                }
                .map_err(|e| e.to_string())?;
            }
            for (key, position) in document.layout().nodes_iter() {
                if document.machine().get_node(key).is_none()
                    || !position.x.is_finite()
                    || !position.y.is_finite()
                {
                    return Err("Invalid node layout entry".into());
                }
            }
            for (_, note) in document.layout().notes_iter() {
                if !note.position.x.is_finite() || !note.position.y.is_finite() {
                    return Err("Invalid note position".into());
                }
            }
        }
        Ok(())
    }

    pub(crate) fn from_parts(
        alphabet: ValidatedAlphabet,
        submachines: SlotMap<SubmachineKey, SubmachineDocument>,
        entry_submachine: SubmachineKey,
    ) -> Self {
        Self {
            alphabet,
            submachines,
            entry_submachine,
        }
    }
}
