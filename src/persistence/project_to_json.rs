use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::model::{
        document::{DiagramDocument, SubmachineDocument},
        layout::LayoutPos,
        node::{Direction, NodeAction},
        submachine::SubmachineName,
        transition::TransitionSymbols,
    },
    project::{Project, model::ProjectName},
};
use serde::Deserialize;
use slotmap::SlotMap;
use std::num::NonZeroU32;

#[derive(Deserialize)]
pub struct ProjectBundle {
    name: String,
    submachines: Vec<Machine>,
    alphabet: Vec<char>,
}
#[derive(Deserialize)]
struct Machine {
    name: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    #[serde(default)]
    notes: Vec<Note>,
}
#[derive(Deserialize)]
struct Node {
    action: Action,
    position: Position,
}
#[derive(Deserialize)]
struct Edge {
    chars: Vec<char>,
    #[serde(default)]
    inverse: bool,
    source: usize,
    target: usize,
}
#[derive(Deserialize)]
struct Note {
    content: String,
    position: Position,
}
#[derive(Clone, Copy, Deserialize)]
struct Position {
    x: f32,
    y: f32,
}
impl From<Position> for LayoutPos {
    fn from(p: Position) -> Self {
        Self::new(p.x, p.y)
    }
}
#[derive(Deserialize)]
enum Action {
    Start,
    Stop,
    Left(u32),
    Right(u32),
    Write(char),
    Submachine { target_id: usize, power: u32 },
}
impl ProjectBundle {
    pub fn into_project(self) -> Result<Project, String> {
        let alphabet = ValidatedAlphabet::new(self.alphabet).map_err(|e| e.to_string())?;
        let name = ProjectName::new(self.name).map_err(|e| e.to_string())?;
        let mut machines = SlotMap::with_key();
        let mut keys = Vec::new();
        for machine in &self.submachines {
            keys.push(machines.insert(SubmachineDocument::new(
                SubmachineName::parse(machine.name.clone()).map_err(|e| e.to_string())?,
            )));
        }
        let entry = *keys.first().ok_or("Project has no main machine")?;
        for (index, machine) in self.submachines.into_iter().enumerate() {
            let document = machines.get_mut(keys[index]).ok_or("Missing submachine")?;
            let mut nodes = Vec::new();
            for node in machine.nodes {
                let action = match node.action {
                    Action::Start => NodeAction::Start,
                    Action::Stop => NodeAction::Stop,
                    Action::Write(c) => NodeAction::Write(c),
                    Action::Left(power) => NodeAction::Move {
                        direction: Direction::Left,
                        repeat: NonZeroU32::new(power).ok_or("Repeat must be positive")?,
                    },
                    Action::Right(power) => NodeAction::Move {
                        direction: Direction::Right,
                        repeat: NonZeroU32::new(power).ok_or("Repeat must be positive")?,
                    },
                    Action::Submachine { target_id, power } => NodeAction::Call {
                        target: *keys.get(target_id).ok_or("Missing called submachine")?,
                        repeat: NonZeroU32::new(power).ok_or("Repeat must be positive")?,
                    },
                };
                nodes.push(document.add_node(node.position.into(), action));
            }
            for edge in machine.edges {
                let source = *nodes.get(edge.source).ok_or("Missing source node")?;
                let target = *nodes.get(edge.target).ok_or("Missing target node")?;
                let symbols = if edge.inverse {
                    TransitionSymbols::new_exclusions(edge.chars, &alphabet)
                } else {
                    TransitionSymbols::new(edge.chars, &alphabet)
                }
                .map_err(|e| e.to_string())?;
                let key = document
                    .add_edge(symbols, source, target)
                    .ok_or("Invalid edge")?;
                document
                    .machine_mut()
                    .get_edge_mut(key)
                    .ok_or("Missing imported edge")?
                    .set_inverse(edge.inverse);
            }
            for note in machine.notes {
                document
                    .layout_mut()
                    .add_note(note.position.into(), note.content);
            }
        }
        let project = Project::new(name, DiagramDocument::from_parts(alphabet, machines, entry));
        Project::validate(project.data())?;
        Ok(project)
    }
}
