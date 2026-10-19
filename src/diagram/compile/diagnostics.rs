use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::model::{
        node::NodeAction,
        submachine::{NodeKey, Submachine},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeProblem {
    NotAvailable,
    MissingTransition(char),
    AmbiguousTransition(char),
    StartNodeIncoming,
    EndNodeOutgoing,
}

pub fn node_problems(
    machine: &Submachine,
    alphabet: &ValidatedAlphabet,
    key: NodeKey,
) -> Vec<NodeProblem> {
    let Some(node) = machine.get_node(key) else {
        return Vec::new();
    };
    let has_incoming = machine.incoming_edges(key).next().is_some();
    let has_outgoing = machine.outgoing_edges(key).next().is_some();
    let mut problems = Vec::new();

    if matches!(node.action(), NodeAction::Stop) {
        if !has_incoming {
            problems.push(NodeProblem::NotAvailable);
        }
        if has_outgoing {
            problems.push(NodeProblem::EndNodeOutgoing);
        }
        return problems;
    }
    if !has_incoming && !has_outgoing {
        problems.push(NodeProblem::NotAvailable);
    }
    if matches!(node.action(), NodeAction::Start) && has_incoming {
        problems.push(NodeProblem::StartNodeIncoming);
    }
    for symbol in alphabet.iter().copied() {
        match machine
            .outgoing_edges(key)
            .filter(|edge| edge.accepts(symbol, alphabet))
            .count()
        {
            0 => problems.push(NodeProblem::MissingTransition(symbol)),
            1 => {}
            _ => problems.push(NodeProblem::AmbiguousTransition(symbol)),
        }
    }
    problems
}
