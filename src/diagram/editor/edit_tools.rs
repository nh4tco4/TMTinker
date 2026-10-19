use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::editor::session::SelectableKey,
    diagram::model::{document::SubmachineDocument, layout::NoteKey},
    diagram::model::{submachine::NodeKey, transition::TransitionSymbols},
};

pub fn delete_tool(submachine_document: &mut SubmachineDocument, selected_item: SelectableKey) {
    match selected_item {
        SelectableKey::Node(key) => submachine_document.remove_node(key),
        SelectableKey::Edge(key) => submachine_document.remove_edge(key),
        SelectableKey::Note(key) => submachine_document.layout_mut().remove_note(key),
    }
}

pub fn link_tool(
    document: &mut SubmachineDocument,
    link_source: &mut Option<NodeKey>,
    alphabet: &ValidatedAlphabet,
    clicked: Option<SelectableKey>,
) {
    let Some(SelectableKey::Node(clicked_node)) = clicked else {
        return;
    };

    match link_source.take() {
        None => {
            *link_source = Some(clicked_node);
        }

        Some(source) => {
            let target = clicked_node;

            let symbols = TransitionSymbols::new(alphabet.clone_iter(), alphabet).unwrap();

            document.machine_mut().add_edge(symbols, source, target);
        }
    }
}

pub fn note_tool(
    submachine: &mut SubmachineDocument,
    world_pos: crate::diagram::model::layout::LayoutPos,
) -> NoteKey {
    submachine
        .layout_mut()
        .add_note(world_pos, "New Note".to_string())
}
