use serde_json::Error;
use slotmap::SecondaryMap;

use crate::{
    core::tm::{self, NodeKey},
    editor::layout,
    project::Project,
    core::tm::SubmachineKey
};

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ProjectBundle {
    pub name: String,
    pub submachines: Vec<Submachine>,
    pub alphabet: Vec<char>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Action {
    Start,
    Stop,
    Left(u32),
    Right(u32),
    Write(char),
    Submachine {
        target_id: usize,
        power: u32,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub(crate) struct Submachine {
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub notes: Vec<Note>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Node {
    pub action: Action,
    pub position: Position,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Edge {
    pub chars: Vec<char>,
    pub source: usize,
    pub target: usize,
}

#[derive(serde::Serialize, serde::Deserialize,)]
pub(crate) struct Note {
    pub content: String,
    pub position: Position,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub(crate) struct Position {
    x: f32,
    y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<layout::LayoutPos> for Position {
    fn from(pos: layout::LayoutPos) -> Self {
        Self::new(pos.x, pos.y)
    }
}

impl From<Position> for layout::LayoutPos {
    fn from(pos: Position) -> Self {
        layout::LayoutPos {
            x: pos.x,
            y: pos.y,
        }
    }
}

impl From<egui::Pos2> for Position {
    fn from(pos: egui::Pos2) -> Self {
        Self::new(pos.x, pos.y)
    }
}

impl From<Position> for egui::Pos2 {
    fn from(pos: Position) -> Self {
        egui::Pos2 {
            x: pos.x,
            y: pos.y,
        }
    }
}

impl ProjectBundle {
    pub fn get_bundle(project: &Project) -> Self {
        let mut bundle = ProjectBundle::default();

        bundle.alphabet = project.get_alphabet_iter().copied().collect();
        bundle.name = project.get_name();

        let mut submachine_id_map: SecondaryMap<SubmachineKey, usize> =
            SecondaryMap::new();

        for (idx, (key, _)) in project.list_submachines().enumerate() {
            submachine_id_map.insert(key, idx);
        }

        for (_, document) in project.list_submachines() {
            let mut submachine = Submachine::default();

            submachine.name = document.machine.name();

            for (_, note) in document.layout.notes_iter() {
                submachine.notes.push(Note {
                    content: note.content.clone(),
                    position: note.position.into(),
                });
            }

            let mut node_id_map: SecondaryMap<NodeKey, usize> =
                SecondaryMap::new();

            for (idx, (key, node)) in document.machine.nodes_iter().enumerate() {
                let action = match &node.action {
                    tm::Action::Start => Action::Start,
                    tm::Action::Stop => Action::Stop,
                    tm::Action::Left(n) => Action::Left(*n),
                    tm::Action::Right(n) => Action::Right(*n),
                    tm::Action::Write(c) => Action::Write(*c),
                    tm::Action::Submachine {
                        key,
                        name: _name,
                        power,
                    } => {
                        let id = *submachine_id_map
                            .get(*key)
                            .expect("Submachine action references missing submachine");

                        Action::Submachine {
                            target_id: id,
                            power: *power,
                        }
                    }
                };

                submachine.nodes.push(Node {
                    action,
                    position: document
                        .layout
                        .get_node_position(key)
                        .expect("Node must have a layout position")
                        .into(),
                });

                node_id_map.insert(key, idx);
            }

            for (_, edge) in document.machine.edges_iter() {
                submachine.edges.push(Edge {
                    chars: edge.chars.iter().collect(),

                    source: *node_id_map
                        .get(edge.source)
                        .expect("Edge source must exist"),

                    target: *node_id_map
                        .get(edge.target)
                        .expect("Edge target must exist"),
                });
            }

            bundle.submachines.push(submachine);
        }

        bundle
    }

    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self)
    }
}
