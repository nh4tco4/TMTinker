use egui::{Id, TextEdit, text::{CCursor, CCursorRange}};

use crate::{core::{ValidatedAlphabet, tm::{Edge, EdgeKey, Submachine, TransitionSymbols, TransitionSymbolsError}}, editor::session::ViewportState};

#[derive(Debug, Clone)]
struct EdgeEditState {
    buffer: String,
    select_all: bool,
    error: Option<TransitionSymbolsError>,
}

pub fn edge_options_base(
    viewport: &ViewportState,
    submachine: &mut Submachine,
    alphabet: &ValidatedAlphabet,
    ctx: &egui::Context,
    edge_key: EdgeKey,
    window_pos: egui::Pos2,
) {

    egui::Window::new("Edge Options")
        .id(Id::new(("edge_options", edge_key)))
        .title_bar(true)
        .resizable(false)
        .default_pos(window_pos)
        .movable(true)
        .scroll_bar_visibility(
            egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
        )
        .show(ctx, |ui| {
            if let Some(edge) = submachine.get_edge_mut(edge_key) {
                choose_symbols(edge, edge_key, ui, edge.chars.clone(), alphabet);
            }
        });
}

fn choose_symbols(
    edge: &mut Edge,
    edge_key: EdgeKey,
    ui: &mut egui::Ui,
    symbols: TransitionSymbols,
    alphabet: &ValidatedAlphabet,
    // window_width: f32,
) {
    let state_id = Id::new(("node_power_edit", edge_key));

    let mut state = ui.ctx().data_mut(|data| {
        data.get_temp::<EdgeEditState>(state_id)
            .unwrap_or_else(|| EdgeEditState {
                buffer: symbols.iter().collect(),
                select_all: true,
                error: None,
            })
    });

    ui.label("Transition signs");

    let response = ui.add(
        TextEdit::singleline(&mut state.buffer)
    );

    if state.select_all {
        select_all(ui.ctx(), response.id, state.buffer.chars().count());
        state.select_all = false;
    }

    if response.changed() {
        let parse_result = TransitionSymbols::new(
                state.buffer.chars(),
                alphabet
            );

        match parse_result {
            Ok(value) => {
                edge.chars = value;
                state.error = None;
            }
            Err(e) => { state.error = Some(e) }
        }
    }

    if let Some(e) = state.error.as_ref() {
        ui.label(format!("{:?}", e));
    }

    ui.ctx().data_mut(|data| {
        data.insert_temp(state_id, state);
    });
}

fn select_all(
    ctx: &egui::Context,
    edit_id: Id,
    char_count: usize,
) {
    if let Some(mut state) = TextEdit::load_state(ctx, edit_id) {
        state.cursor.set_char_range(Some(
            CCursorRange::two(
                CCursor::new(0),
                CCursor::new(char_count),
            ),
        ));

        state.store(ctx, edit_id);
    }
}
