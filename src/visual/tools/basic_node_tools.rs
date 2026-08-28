use egui::Pos2;

use crate::{core::tm::{Action, NodeKey, SubmachineKey}, project::SubmachineDocument};

pub fn place_node(document: &mut SubmachineDocument, position: Pos2, action: Action) -> NodeKey {
    document.add_node(position, action)
}

pub fn left_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(document, position, Action::Left(1))
}

pub fn right_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(document, position, Action::Right(1))
}

pub fn write_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(document, position, Action::Write(' '))
}

pub fn start_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    // let existing: Vec<_> = document
    //     .iter()
    //     .filter(|(_, n)| n.action == Action::Start)
    //     .map(|(k, _)| k)
    //     .collect();
    // for key in existing {
    //     document.current_graph_mut().remove_node(key);
    // }
    place_node(document, position, Action::Start)
}

pub fn stop_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(document, position, Action::Stop)
}

pub fn submachine_tool(document: &mut SubmachineDocument, position: Pos2, submachine_key: &SubmachineKey, name: String) -> NodeKey {
    place_node(document, position, Action::Submachine {name, key: *submachine_key, power: 1})
}
