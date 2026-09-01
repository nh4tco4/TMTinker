use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::{
    diagram::model::{
        node::{Node, NodeAction},
        transition::{Edge, EdgeKey, TransitionSymbols},
    },
    project::alphabet::ValidatedAlphabet,
};

new_key_type! {
    pub struct SubmachineKey;
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MachineState {
    Stopped,
    Running,
    Halted(HaltReason),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HaltReason {
    EndAction,
    NoTransition,
    InvalidStartState,
    SubmachineReturned,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StepResult {
    Executed(usize),
    Halted(HaltReason),
    Error(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubmachineFrame {
    parent_submachine_id: usize,
    parent_state: Option<NodeKey>,
}

new_key_type! {
    pub struct NodeKey;
}
