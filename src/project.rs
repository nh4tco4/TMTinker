use slotmap::SlotMap; use crate::{app::ProjectKey, core::{ValidatedAlphabet, tm::{Action, NodeKey, Submachine, SubmachineKey, TransitionSymbols}}, editor::{layout::GraphLayout, session::ViewportState}, persistence::{project_to_json::{self, ProjectBundle}}}; pub enum ProjectAction { LoadFromMemory(ProjectKey), LoadFromJson(ProjectKey), LoadFromRam(ProjectKey), CreateProject((String, ValidatedAlphabet)),
    CloseProject
}

pub struct SubmachineDocument {
    pub machine: Submachine,
    pub layout: GraphLayout,
    pub viewport: ViewportState,
}

impl SubmachineDocument {
    pub fn new(submachine_name: String) -> Self {
        Self {
            machine: Submachine::new(submachine_name),
            layout: GraphLayout::new(),
            viewport: ViewportState::new(),
        }
    }

    pub fn add_node(
        &mut self,
        position: egui::Pos2,
        node_action: Action,
    ) -> NodeKey {
        let node_key = self.machine.add_node(node_action);

        self.layout.set_node_position(
            node_key,
            position.into(),
        );

        node_key
    }

    pub fn remove_node(&mut self, node_key: NodeKey) {
        self.machine.remove_node(node_key);
        self.layout.remove_node(node_key);
    }
}

pub struct Project {
    name: String,
    current_submachine_id: SubmachineKey,
    submachines: SlotMap<SubmachineKey, SubmachineDocument>,
    entry_submachine: SubmachineKey,
    alphabet: ValidatedAlphabet
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveSubmachineError {
    #[error("Cannot remove the current active submachine.")]
    CannotRemoveCurrentMachine,

    #[error("Cannot remove the main machine.")]
    CannotRemoveMainMachine,
}

impl Project {
    pub fn new(project_name: String, alphabet: ValidatedAlphabet) -> Self {
        let mut submachines = SlotMap::with_key();
        let new_id = submachines.insert(SubmachineDocument::new("Main machine".to_string()));

        Self {
            name: project_name,
            current_submachine_id: new_id,
            entry_submachine: new_id,
            submachines,
            alphabet,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn current_submachine_key(
        &self,
    ) -> &SubmachineKey {
        &self.current_submachine_id
    }

    pub fn current_document_mut(
        &mut self,
    ) -> &mut SubmachineDocument {
        &mut self.submachines[self.current_submachine_id]
    }

    pub fn set_submachine(
        &mut self,
        key: SubmachineKey,
    ) {
        self.current_submachine_id = key;
    }

    pub fn add_submachine(&mut self, name: String) {
        self.current_submachine_id = self.submachines.insert(SubmachineDocument::new(name));
    }

    pub fn remove_submachine(&mut self, id: SubmachineKey) -> Result<(), RemoveSubmachineError> {
        if id == self.current_submachine_id {
            return Err(RemoveSubmachineError::CannotRemoveCurrentMachine);
        } else if id == self.entry_submachine {
            return Err(RemoveSubmachineError::CannotRemoveMainMachine);
        }

        // TODO warn about machine being part of other machines

        let _ = self.submachines.remove(id);
        Ok(())
    }

    pub fn get_alphabet_iter(&self) -> impl Iterator<Item = &char> {
        self.alphabet.iter()
    }

    pub fn get_alphabet(&self) -> ValidatedAlphabet {
        self.alphabet.clone()
    }

    pub fn rename_project(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn info(&self) -> (String, impl Iterator<Item = &char>) {
        (self.name.clone(), self.alphabet.iter())
    }

    pub fn list_submachines(&self) -> impl Iterator<Item = (SubmachineKey, &SubmachineDocument)> {
        self.submachines.iter()
    }

    pub fn validate_machines(&mut self) {
        for (_, document) in self.submachines.iter_mut() {
            document.machine.validate(self.alphabet.clone());
        }
    }

    pub fn from_json(json: String) -> Result<Project, serde_json::Error> {
        let bundle: ProjectBundle = serde_json::from_str(&json)?;

        let validated_alphabet =
            ValidatedAlphabet::new(bundle.alphabet.clone())
                .expect("couldn't validate alphabet");

        let mut submachines: SlotMap<SubmachineKey, SubmachineDocument> =
            SlotMap::with_key();

        let mut submachine_key_name: Vec<(SubmachineKey, String)> =
            Vec::with_capacity(bundle.submachines.len());

        for submachine in &bundle.submachines {
            let key = submachines.insert(
                SubmachineDocument::new(submachine.name.clone())
            );

            submachine_key_name.push((
                key,
                submachine.name.clone(),
            ));
        }

        let entry_submachine = submachine_key_name
            .first()
            .map(|(key, _)| *key)
            .expect("project must contain at least one submachine");

        let mut node_keys: Vec<Vec<NodeKey>> =
            Vec::with_capacity(bundle.submachines.len());

        for (machine_index, serialized_machine) in
            bundle.submachines.iter().enumerate()
        {
            let machine_key = submachine_key_name[machine_index].0;

            let document = &mut submachines[machine_key];

            let mut machine_node_keys =
                Vec::with_capacity(serialized_machine.nodes.len());

            for serialized_node in &serialized_machine.nodes {
                let action = match &serialized_node.action {
                    project_to_json::Action::Start => Action::Start,
                    project_to_json::Action::Stop => Action::Stop,
                    project_to_json::Action::Left(n) => Action::Left(*n),
                    project_to_json::Action::Right(n) => Action::Right(*n),
                    project_to_json::Action::Write(c) => Action::Write(*c),
                    project_to_json::Action::Submachine {
                        target_id,
                        power
                    } => {
                        let (key, name) =
                            &submachine_key_name[*target_id];

                        Action::Submachine {
                            name: name.clone(),
                            key: *key,
                            power: *power,
                        }
                    }
                };

                let node_key =
                    document.machine.add_node(action);

                document.layout.set_node_position(
                    node_key,
                    serialized_node.position.into(),
                );

                machine_node_keys.push(node_key);
            }

            node_keys.push(machine_node_keys);
        }

        for (machine_index, serialized_machine) in
            bundle.submachines.iter().enumerate()
        {
            let machine_key =
                submachine_key_name[machine_index].0;

            let document =
                &mut submachines[machine_key];

            for serialized_edge in &serialized_machine.edges {
                let source =
                    node_keys[machine_index]
                        [serialized_edge.source];

                let target =
                    node_keys[machine_index]
                        [serialized_edge.target];

                document.machine.add_edge(
                    TransitionSymbols::new(
                        serialized_edge.chars.clone(),
                        &validated_alphabet,
                    ).unwrap(),
                // TODO ^- remove this unwrap with proper error handeling
                // or i can move responsibility for that to validating machine
                    source,
                    target,
                );
            }
        }

        for (machine_index, serialized_machine) in
            bundle.submachines.iter().enumerate()
        {
            let machine_key =
                submachine_key_name[machine_index].0;

            let document =
                &mut submachines[machine_key];

            for serialized_note in &serialized_machine.notes {
                document.layout.add_note(
                    serialized_note.position.into(),
                    serialized_note.content.clone(),
                );
            }
        }

        let mut project = Project {
            name: bundle.name,
            current_submachine_id: entry_submachine,
            entry_submachine,
            submachines,
            alphabet: validated_alphabet,
        };

        project.validate_machines();

        Ok(project)
    }
}
