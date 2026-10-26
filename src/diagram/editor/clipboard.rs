use std::collections::{HashMap, HashSet};

use super::canvas_state::{SelectableKey, SelectionState};
use crate::diagram::model::{
    document::SubmachineDocument,
    layout::{GraphLayout, LayoutPos, Note},
    node::NodeAction,
    submachine::NodeKey,
    transition::Edge,
};

// The actual data stays in the editor session, so it cannot cross project boundaries.
const CLIPBOARD_MARKER: &str = "TMTinker canvas selection";

pub struct CanvasClipboard {
    nodes: Vec<(NodeKey, NodeAction, LayoutPos)>,
    notes: Vec<Note>,
    edges: Vec<Edge>,
    offset: LayoutPos,
}

impl CanvasClipboard {
    fn copy(document: &SubmachineDocument, selection: &SelectionState) -> Option<Self> {
        let nodes: Vec<_> = document
            .machine()
            .nodes_iter()
            .filter(|(key, _)| selection.is_selected(&SelectableKey::Node(*key)))
            .filter_map(|(key, node)| {
                Some((
                    key,
                    node.action().clone(),
                    document.layout().get_node_position(key)?,
                ))
            })
            .collect();
        let notes: Vec<_> = document
            .layout()
            .notes_iter()
            .filter(|(key, _)| selection.is_selected(&SelectableKey::Note(*key)))
            .map(|(_, note)| note.clone())
            .collect();
        if nodes.is_empty() && notes.is_empty() {
            return None;
        }
        let copied_keys: HashSet<_> = nodes.iter().map(|(key, ..)| *key).collect();
        let edges = document
            .machine()
            .edges_iter()
            .filter(|(_, edge)| {
                copied_keys.contains(&edge.source()) && copied_keys.contains(&edge.target())
            })
            .map(|(_, edge)| edge.clone())
            .collect();
        Some(Self {
            nodes,
            notes,
            edges,
            offset: LayoutPos::new(0., 0.),
        })
    }

    fn paste(&mut self, document: &mut SubmachineDocument, selection: &mut SelectionState) {
        self.offset += LayoutPos::new(GraphLayout::GRID_SIZE, GraphLayout::GRID_SIZE);
        selection.clear();
        let mut keys = HashMap::new();
        for (old_key, action, position) in &self.nodes {
            let key = document.add_node(*position + self.offset, action.clone());
            keys.insert(*old_key, key);
            selection.selected.push(SelectableKey::Node(key));
        }
        for note in &self.notes {
            let key = document
                .layout_mut()
                .add_note(note.position + self.offset, note.content.clone());
            selection.selected.push(SelectableKey::Note(key));
        }
        for edge in &self.edges {
            if let Some(key) = document.add_edge(
                edge.symbols().clone(),
                *keys.get(&edge.source()).expect("copied edge source"),
                *keys.get(&edge.target()).expect("copied edge target"),
            ) {
                document
                    .machine_mut()
                    .get_edge_mut(key)
                    .expect("new edge")
                    .set_inverse(edge.inverse());
                selection.selected.push(SelectableKey::Edge(key));
            }
        }
    }
}

pub fn handle_clipboard(
    clipboard: &mut Option<CanvasClipboard>,
    document: &mut SubmachineDocument,
    selection: &mut SelectionState,
    ctx: &egui::Context,
) {
    if ctx.wants_keyboard_input() || ctx.memory(|m| m.top_modal_layer().is_some()) {
        return;
    }
    let (copy, paste) = ctx.input_mut(|input| {
        // eframe translates the platform shortcuts into these events. Acting on
        // the raw key as well can paste twice when the browser delivers it first.
        let mut copy = false;
        let mut paste = false;
        input.events.retain(|event| match event {
            egui::Event::Copy => {
                copy = true;
                false
            }
            egui::Event::Paste(text) if text == CLIPBOARD_MARKER => {
                paste = true;
                false
            }
            _ => true,
        });
        (copy, paste)
    });
    if copy {
        *clipboard = CanvasClipboard::copy(document, selection);
        if clipboard.is_some() {
            ctx.copy_text(CLIPBOARD_MARKER.into());
        }
    }
    if paste && let Some(clipboard) = clipboard {
        clipboard.paste(document, selection);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CanvasClipboard, GraphLayout, LayoutPos, NodeAction, SelectableKey, SelectionState,
    };
    use crate::{
        core::alphabet::ValidatedAlphabet,
        diagram::{
            compile::compile,
            model::{document::DiagramDocument, submachine::SubmachineName},
        },
    };

    #[test]
    fn call_only_selection_pastes_shared_references_in_any_submachine() {
        let mut document =
            DiagramDocument::new(ValidatedAlphabet::new([' ', '0', '1']).expect("alphabet"));
        let main = document.entry_submachine();
        let child = document.add_submachine(SubmachineName::parse("L".into()).expect("name"));
        let other = document.add_submachine(SubmachineName::parse("Other".into()).expect("name"));
        let action = NodeAction::Call {
            target: child,
            repeat: 3.try_into().expect("positive"),
        };
        let position = LayoutPos::new(150., 225.);
        let machine = document.submachine_mut(main).expect("main");
        let original = machine.add_node(position, action.clone());
        let mut selection = SelectionState::default();
        selection.selected.push(SelectableKey::Node(original));
        let mut clipboard = CanvasClipboard::copy(machine, &selection).expect("copy call alone");

        for (index, destination) in [main, other, child].into_iter().enumerate() {
            let machine = document.submachine_mut(destination).expect("destination");
            clipboard.paste(machine, &mut selection);
            let [SelectableKey::Node(pasted)] = selection.selected.as_slice() else {
                panic!("paste should select the new call node");
            };
            assert_eq!(
                machine
                    .machine()
                    .get_node(*pasted)
                    .expect("pasted")
                    .action(),
                &action
            );
            assert_eq!(
                machine.layout().get_node_position(*pasted),
                Some(
                    position
                        + LayoutPos::new(
                            GraphLayout::GRID_SIZE * (index + 1) as f32,
                            GraphLayout::GRID_SIZE * (index + 1) as f32,
                        )
                )
            );
            if destination == main {
                assert_ne!(*pasted, original);
                assert_eq!(machine.machine().nodes_iter().count(), 2);
            }
            document.validate().expect("editable document");
            if destination != child {
                document
                    .validate_calls()
                    .expect("shared calls are not recursive");
            }
        }
        assert_eq!(document.submachines().count(), 3);
        assert!(
            compile(&document)
                .err()
                .expect("recursive call")
                .to_string()
                .contains("L -> L")
        );
    }
}
