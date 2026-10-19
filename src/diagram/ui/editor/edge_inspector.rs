use egui::{
    Id, TextEdit,
    text::{CCursor, CCursorRange},
};

use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::editor::session::ViewportState,
    diagram::model::{
        submachine::Submachine,
        transition::{Edge, EdgeKey, TransitionSymbols, TransitionSymbolsError},
    },
};

#[derive(Debug, Clone)]
struct EdgeEditState {
    buffer: String,
    select_all: bool,
    error: Option<TransitionSymbolsError>,
    committed_symbols: TransitionSymbols,
    committed_inverse: bool,
}

#[derive(Default)]
pub struct EdgeInspectorState {
    edits: std::collections::HashMap<EdgeKey, EdgeEditState>,
}

pub fn edge_options_base(
    _viewport: &ViewportState,
    submachine: &mut Submachine,
    alphabet: &ValidatedAlphabet,
    ctx: &egui::Context,
    edge_key: EdgeKey,
    window_pos: egui::Pos2,
    state: &mut EdgeInspectorState,
) {
    egui::Window::new("Edge Options")
        .id(Id::new(("edge_options", edge_key)))
        .title_bar(true)
        .resizable(false)
        .default_pos(window_pos)
        .movable(true)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ctx, |ui| {
            if let Some(edge) = submachine.get_edge_mut(edge_key) {
                choose_symbols(edge, edge_key, ui, edge.symbols().clone(), alphabet, state);
            }
        });
}

fn choose_symbols(
    edge: &mut Edge,
    edge_key: EdgeKey,
    ui: &mut egui::Ui,
    symbols: TransitionSymbols,
    alphabet: &ValidatedAlphabet,
    inspector: &mut EdgeInspectorState,
) {
    let state = inspector
        .edits
        .entry(edge_key)
        .or_insert_with(|| EdgeEditState {
            buffer: alphabet
                .iter()
                .copied()
                .filter(|&symbol| symbols.contains(symbol))
                .map(|symbol| if symbol == ' ' { 'λ' } else { symbol })
                .collect(),
            select_all: true,
            error: None,
            committed_symbols: symbols.clone(),
            committed_inverse: edge.inverse(),
        });

    if state.committed_symbols != symbols || state.committed_inverse != edge.inverse() {
        state.buffer = alphabet
            .iter()
            .copied()
            .filter(|&symbol| symbols.contains(symbol))
            .map(|symbol| if symbol == ' ' { 'λ' } else { symbol })
            .collect();
        state.committed_symbols = symbols;
        state.committed_inverse = edge.inverse();
        state.error = None;
    }

    ui.label("Transition signs");
    ui.weak("λ = blank cell");

    let mut inverse = edge.inverse();
    let inverse_changed = ui.toggle_value(&mut inverse, "Inverse").changed();
    if inverse {
        ui.weak("Accept every alphabet symbol except those below.");
    }
    let response = ui.add(TextEdit::singleline(&mut state.buffer));

    if state.select_all {
        select_all(ui.ctx(), response.id, state.buffer.chars().count());
        state.select_all = false;
    }

    if response.changed() || inverse_changed {
        let parse_result = parse_condition_symbols(&state.buffer, alphabet, inverse);

        match parse_result {
            Ok(value) => {
                state.committed_symbols = value.clone();
                state.committed_inverse = inverse;
                edge.set_symbols(value);
                edge.set_inverse(inverse);
                state.error = None;
            }
            Err(e) => state.error = Some(e),
        }
    }

    if let Some(e) = state.error.as_ref() {
        ui.colored_label(egui::Color32::DARK_RED, e.to_string());
    }
}

fn parse_condition_symbols(
    buffer: &str,
    alphabet: &ValidatedAlphabet,
    inverse: bool,
) -> Result<TransitionSymbols, TransitionSymbolsError> {
    let symbols = buffer
        .chars()
        .map(|symbol| if symbol == 'λ' { ' ' } else { symbol });
    if inverse {
        TransitionSymbols::new_exclusions(symbols, alphabet)
    } else {
        TransitionSymbols::new(symbols, alphabet)
    }
}

fn select_all(ctx: &egui::Context, edit_id: Id, char_count: usize) {
    if let Some(mut state) = TextEdit::load_state(ctx, edit_id) {
        state.cursor.set_char_range(Some(CCursorRange::two(
            CCursor::new(0),
            CCursor::new(char_count),
        )));

        state.store(ctx, edit_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tape::Tape;
    use crate::diagram::{
        compile::compile,
        model::{document::DiagramDocument, layout::LayoutPos, node::NodeAction},
        runtime::{DiagramRunSession, RunStatus},
    };

    fn parse_symbols(
        buffer: &str,
        alphabet: &ValidatedAlphabet,
    ) -> Result<TransitionSymbols, TransitionSymbolsError> {
        parse_condition_symbols(buffer, alphabet, false)
    }

    #[test]
    fn lambda_in_transition_field_runs_on_a_blank_tape() {
        let alphabet = ValidatedAlphabet::new([' ', '0', '1']).unwrap();
        let mut document = DiagramDocument::new(alphabet.clone());
        let machine = document
            .submachine_mut(document.entry_submachine())
            .unwrap();
        let start = machine.add_node(LayoutPos::new(0., 0.), NodeAction::Start);
        let stop = machine.add_node(LayoutPos::new(75., 0.), NodeAction::Stop);
        let edge = machine
            .add_edge(parse_symbols("λ01", &alphabet).unwrap(), start, stop)
            .unwrap();
        let compiled =
            compile(&document).expect("visible blank symbol covers the blank transition");
        let mut session = DiagramRunSession::new(compiled, Tape::new());
        session.step();
        session.step();
        assert_eq!(session.status(), RunStatus::Halted);
        assert_eq!(session.tape(), &Tape::new());

        document
            .submachine_mut(document.entry_submachine())
            .unwrap()
            .machine_mut()
            .get_edge_mut(edge)
            .unwrap()
            .set_symbols(parse_symbols("01", &alphabet).unwrap());
        let error = compile(&document)
            .err()
            .expect("missing blank transition must still fail");
        assert!(error.to_string().contains("'λ'"));
        assert_eq!(
            parse_symbols(" 01", &alphabet).unwrap(),
            parse_symbols("λ01", &alphabet).unwrap()
        );
    }
}
