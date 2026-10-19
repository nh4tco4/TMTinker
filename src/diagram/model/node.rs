use crate::diagram::model::submachine::SubmachineKey;
use std::num::NonZeroU32;

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub enum Direction {
    Left,
    Right,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub enum NodeAction {
    Start,
    Stop,
    Move {
        direction: Direction,
        repeat: NonZeroU32,
    },
    Write(char),
    Call {
        target: SubmachineKey,
        repeat: NonZeroU32,
    },
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Node {
    action: NodeAction,
}

impl Node {
    pub fn action(&self) -> &NodeAction {
        &self.action
    }

    pub fn set_action(&mut self, new_action: NodeAction) {
        self.action = new_action;
    }

    pub fn new(action: NodeAction) -> Self {
        Self { action }
    }
}
