use slotmap::{SlotMap, new_key_type};

use crate::diagram::model::{
    node::{Node, NodeAction},
    transition::{Edge, EdgeKey, TransitionSymbols},
};

new_key_type! {
    pub struct SubmachineKey;
    pub struct NodeKey;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Submachine {
    name: SubmachineName,
    nodes: SlotMap<NodeKey, Node>,
    edges: SlotMap<EdgeKey, Edge>,
}

impl PartialEq for Submachine {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.nodes.iter().eq(other.nodes.iter())
            && self.edges.iter().eq(other.edges.iter())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SubmachineName(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SubmachineNameError {
    #[error("Submachine name cannot be empty.")]
    Empty,

    #[error("Submachine name cannot be longer than 20 characters.")]
    NameTooLong,
}

impl SubmachineName {
    pub fn parse(value: String) -> Result<Self, SubmachineNameError> {
        let value = value.trim();
        let length = value.chars().count();
        if length < 1 {
            return Err(SubmachineNameError::Empty);
        } else if length > 20 {
            return Err(SubmachineNameError::NameTooLong);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Submachine {
    pub fn new(name: SubmachineName) -> Self {
        Self {
            name,
            nodes: SlotMap::with_key(),
            edges: SlotMap::with_key(),
        }
    }

    pub fn add_node(&mut self, action: NodeAction) -> NodeKey {
        self.nodes.insert(Node::new(action))
    }

    pub fn add_edge(
        &mut self,
        chars: TransitionSymbols,
        source: NodeKey,
        target: NodeKey,
    ) -> EdgeKey {
        self.edges.insert(Edge::new(chars, source, target))
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = (NodeKey, &Node)> {
        self.nodes.iter()
    }

    pub fn get_node(&self, key: NodeKey) -> Option<&Node> {
        self.nodes.get(key)
    }

    pub fn get_mut_node(&mut self, key: NodeKey) -> Option<&mut Node> {
        self.nodes.get_mut(key)
    }

    pub fn edges_iter(&self) -> impl Iterator<Item = (EdgeKey, &Edge)> {
        self.edges.iter()
    }

    pub fn get_edge_mut(&mut self, key: EdgeKey) -> Option<&mut Edge> {
        self.edges.get_mut(key)
    }

    pub fn outgoing_edges(&self, node: NodeKey) -> impl Iterator<Item = &Edge> {
        self.edges
            .values()
            .filter(move |edge| edge.source() == node)
    }

    pub fn incoming_edges(&self, node: NodeKey) -> impl Iterator<Item = &Edge> {
        self.edges
            .values()
            .filter(move |edge| edge.target() == node)
    }

    pub fn remove_node(&mut self, key: NodeKey) {
        let edge_keys: Vec<EdgeKey> = self
            .edges
            .iter()
            .filter(|(_, edge)| edge.source() == key || edge.target() == key)
            .map(|(k, _)| k)
            .collect();

        for edge_key in edge_keys {
            self.edges.remove(edge_key);
        }

        self.nodes.remove(key);
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn rename(&mut self, name: SubmachineName) {
        self.name = name;
    }

    pub fn remove_edge(&mut self, key: EdgeKey) {
        self.edges.remove(key);
    }
}
