use egui::Pos2;

use crate::{
    core::NonZeroU32,
    diagram::model::{
        node::{Direction, NodeAction},
        submachine::{NodeKey, SubmachineKey},
    },
    project::SubmachineDocument,
};

pub fn place_node(
    document: &mut SubmachineDocument,
    position: Pos2,
    action: NodeAction,
) -> NodeKey {
    document.add_node(position, action)
}

pub fn left_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(
        document,
        position,
        NodeAction::Move {
            direction: Direction::Left,
            repeat: NonZeroU32::new(1),
        },
    )
}

pub fn right_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(
        document,
        position,
        NodeAction::Move {
            direction: Direction::Right,
            repeat: NonZeroU32::new(1),
        },
    )
}

pub fn write_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(document, position, NodeAction::Write(' '))
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
    place_node(document, position, NodeAction::Start)
}

pub fn stop_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(document, position, NodeAction::Stop)
}

pub fn submachine_tool(
    document: &mut SubmachineDocument,
    position: Pos2,
    submachine_key: &SubmachineKey,
) -> NodeKey {
    place_node(
        document,
        position,
        NodeAction::Call {
            target: *submachine_key,
            repeat: NonZeroU32::new(1),
        },
    )
}
