use std::collections::{HashSet, hash_set::Iter};

use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::core::ValidatedAlphabet;

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
pub struct Submachine {
    name: String,
    nodes: SlotMap<NodeKey, Node>,
    edges: SlotMap<EdgeKey, Edge>,
    problematic_nodes: SecondaryMap<NodeKey, Vec<NodeProblem>>,
}

impl Submachine {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            nodes: SlotMap::with_key(),
            problematic_nodes: SecondaryMap::new(),
            edges: SlotMap::with_key(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubmachineFrame {
    parent_submachine_id: usize,
    parent_state: Option<NodeKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum TransitionSymbolsError {
    #[error("Symbols must not be empty")]
    Empty,

    #[error("Alphabet doesn't contain selected character.")]
    AlphabetDoesNotContain(char),

    #[error("Alphabet contains duplicated characters.")]
    DuplicatedCharacter(char),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TransitionSymbols(HashSet<char>);

pub struct TransitionSymbolsIter<'a> {
    inner: Iter<'a, char>
}

impl<'a> Iterator for TransitionSymbolsIter<'a> {
    type Item = char;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }
}

impl<'a> IntoIterator for &'a TransitionSymbols {
    type Item = char;
    type IntoIter = TransitionSymbolsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl TransitionSymbols {
    pub fn new(
        symbols: impl IntoIterator<Item = char>,
        alphabet: &ValidatedAlphabet,
    ) -> Result<Self, TransitionSymbolsError> {
        let alphabet_set: HashSet<char> = alphabet.iter().copied().collect();
        let mut symbols_set = HashSet::new();

        for symbol in symbols {
            if !alphabet_set.contains(&symbol) {
                return Err(TransitionSymbolsError::AlphabetDoesNotContain(symbol));
            }

            if !symbols_set.insert(symbol) {
                return Err(TransitionSymbolsError::DuplicatedCharacter(symbol));
            }
        }

        if symbols_set.is_empty() {
            return Err(TransitionSymbolsError::Empty);
        }

        Ok(Self(symbols_set))
    }

    pub fn contains(&self, symbol: char) -> bool {
        self.0.contains(&symbol)
    }

    pub fn iter(&self) -> TransitionSymbolsIter<'_> {
        TransitionSymbolsIter {
            inner: self.0.iter()
        }
    }
}

new_key_type! {
    pub struct NodeKey;
    pub struct EdgeKey;
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub enum Action {
    Start,
    Stop,
    Left(u32),
    Right(u32),
    Write(char),
    Submachine {
        name: String,
        key: SubmachineKey,
        power: u32,
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Node {
    pub action: Action,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Edge {
    pub chars: TransitionSymbols,
    pub source: NodeKey,
    pub target: NodeKey,
}

impl Edge {
    pub fn accepts(&self, symbol: char) -> bool {
        self.chars.iter().any(|c| c == symbol)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub enum NodeProblem {
    NotAvailable,
    MissingTransition(char),
    AmbiguousTransition(char),
    StartNodeIncoming,
    EndNodeOutgoing,
}


impl Submachine {
    pub fn add_node(&mut self, action: Action) -> NodeKey {
        self.nodes.insert(Node { action })
    }

    pub fn add_edge(&mut self, chars: TransitionSymbols, source: NodeKey, target: NodeKey) -> EdgeKey {
        self.edges.insert(Edge {
            chars,
            source,
            target,
        })
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

    pub fn get_node_action(&self, key: NodeKey) -> Option<&Action> {
        self.nodes.get(key).map(|node| &node.action)
    }

    pub fn get_node_problems(&self, key: NodeKey) -> Vec<NodeProblem> {
        if let Some(node) = self.problematic_nodes.get(key) {
            node.clone()
        } else {
            vec![]
        }
    }

    pub fn edges_iter(&self) -> impl Iterator<Item = (EdgeKey, &Edge)> {
        self.edges.iter()
    } 

    pub fn get_transition(&self, key: NodeKey, symbol: char) -> Option<NodeKey> {
        self.outgoing_edges(key)
            .find(|edge| edge.chars.contains(symbol))
            .map(|edge| edge.target)
    }

    pub fn get_edge(&self, key: EdgeKey) -> Option<&Edge> {
        self.edges.get(key)
    }

    pub fn get_edge_mut(&mut self, key: EdgeKey) -> Option<&mut Edge> {
        self.edges.get_mut(key)
    }

    pub fn outgoing_edges(&self, node: NodeKey) -> impl Iterator<Item = &Edge> {
        self.edges.values().filter(move |edge| edge.source == node)
    }

    pub fn incoming_edges(&self, node: NodeKey) -> impl Iterator<Item = &Edge> {
        self.edges.values().filter(move |edge| edge.target == node)
    }

    pub fn remove_node(&mut self, key: NodeKey) {
        let edge_keys: Vec<EdgeKey> = self
            .edges
            .iter()
            .filter(|(_, e)| e.source == key || e.target == key)
            .map(|(k, _)| k)
            .collect();

        for edge_key in edge_keys {
            self.edges.remove(edge_key);
        }

        self.nodes.remove(key);
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn remove_edge(&mut self, key: EdgeKey) {
        self.edges.remove(key);
    }

    pub fn validate(&mut self, alphabet: ValidatedAlphabet) {
        self.problematic_nodes.clear();

        for (key, node) in self.nodes.iter() {
            let has_incoming = self.incoming_edges(key).next().is_some();
            let has_outgoing = self.outgoing_edges(key).next().is_some();

            let mut problems = Vec::new();

            match node.action {
                Action::Left(_) | Action::Right(_) | Action::Submachine { .. } | Action::Write(_) => {
                    if !has_incoming && !has_outgoing {
                        problems.push(NodeProblem::NotAvailable);
                    }

                    for symbol in alphabet.iter() {
                        let matching_edges = self.outgoing_edges(key)
                            .filter(|edge| edge.accepts(*symbol))
                            .count();

                        let problem: Option<NodeProblem> = match matching_edges {
                            0 => Some(NodeProblem::MissingTransition(*symbol)),
                            1 => None,
                            _ => Some(NodeProblem::AmbiguousTransition(*symbol)),
                        };

                        if let Some(new_problem) = problem {
                            problems.push(new_problem);
                        }
                    }
                }
                Action::Start => {
                    if !has_outgoing {
                        problems.push(NodeProblem::NotAvailable);
                    }

                    if has_incoming {
                        problems.push(NodeProblem::StartNodeIncoming);
                    }

                    for symbol in alphabet.iter() {
                        let matching_edges = self.outgoing_edges(key)
                            .filter(|edge| edge.accepts(*symbol))
                            .count();

                        let problem: Option<NodeProblem> = match matching_edges {
                            0 => Some(NodeProblem::MissingTransition(*symbol)),
                            1 => None,
                            _ => Some(NodeProblem::AmbiguousTransition(*symbol)),
                        };

                        if let Some(new_problem) = problem {
                            problems.push(new_problem);
                        }
                    }
                }
                Action::Stop => {
                    if !has_incoming {
                        problems.push(NodeProblem::NotAvailable);
                    }

                    if has_outgoing {
                        problems.push(NodeProblem::EndNodeOutgoing);
                    }
                }
            }

            if !problems.is_empty() {
                self.problematic_nodes.insert(key, problems);
            }
        }
    }
}
