use egui::Pos2;
use std::num::NonZeroU32;

use crate::{
    diagram::model::document::SubmachineDocument,
    diagram::model::{
        node::{Direction, NodeAction},
        submachine::{NodeKey, SubmachineKey},
    },
};

pub fn place_node(
    document: &mut SubmachineDocument,
    position: Pos2,
    action: NodeAction,
) -> NodeKey {
    document.add_node(position.into(), action)
}

pub fn left_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(
        document,
        position,
        NodeAction::Move {
            direction: Direction::Left,
            repeat: NonZeroU32::MIN,
        },
    )
}

pub fn right_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(
        document,
        position,
        NodeAction::Move {
            direction: Direction::Right,
            repeat: NonZeroU32::MIN,
        },
    )
}

pub fn write_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
    place_node(document, position, NodeAction::Write(' '))
}

pub fn start_tool(document: &mut SubmachineDocument, position: Pos2) -> NodeKey {
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
            repeat: NonZeroU32::MIN,
        },
    )
}
