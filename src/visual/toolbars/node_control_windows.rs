use egui::{
    Id, TextEdit, text::{CCursor, CCursorRange},
};

use crate::{
    core::{ValidatedAlphabet, tm::{Action, Node, NodeKey, NodeProblem, Submachine}}, editor::{layout::LayoutPos, session::ViewportState}, visual::{
        mappings::{NODE_SIZE, descriptions::*},
        utils::text_input::{buffer_to_load, formatted_buffer},
    },
};

#[derive(Clone)]
struct PowerEditState {
    value: String,
    select_all: bool,
}

#[derive(Clone)]
struct SignEditState {
    value: String,
    select_all: bool,
}

pub fn node_options_base(
    viewport: &ViewportState,
    submachine: &mut Submachine,
    alphabet: &ValidatedAlphabet,
    ctx: &egui::Context,
    node_key: NodeKey,
    window_pos: egui::Pos2,
) {
    let shift =
        LayoutPos::new(NODE_SIZE, NODE_SIZE) * viewport.camera_zoom.get();

    let new_pos = viewport.screen_to_world(shift + window_pos.into());

    egui::Window::new("Node Options")
        .id(Id::new(("node_options", node_key)))
        .title_bar(true)
        .resizable(false)
        .fixed_pos(new_pos)
        .movable(true)
        .scroll_bar_visibility(
            egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
        )
        .show(ctx, |ui| {
            let problems = submachine.get_node_problems(node_key);

            if let Some(node) = submachine.get_mut_node(node_key) {
                let window_width = ui.available_width();

                match node.action {
                    Action::Start => {
                        show_description(ui, start_node_description());
                    }
                    Action::Stop => {
                        show_description(ui, end_node_description());
                    }
                    Action::Left(power) => {
                        choose_power(
                            node,
                            node_key,
                            ui,
                            power,
                            window_width,
                        );

                        ui.separator();
                        show_description(ui, left_node_description());
                    }
                    Action::Right(power) => {
                        choose_power(
                            node,
                            node_key,
                            ui,
                            power,
                            window_width,
                        );

                        ui.separator();
                        show_description(ui, right_node_description());
                    }
                    Action::Submachine {power, .. } => {
                        choose_power(
                            node,
                            node_key,
                            ui,
                            power,
                            window_width,
                        );

                        ui.separator();
                        show_description(ui, submachine_node_description());
                    }
                    Action::Write(character) => {
                        choose_sign(
                            node,
                            node_key,
                            ui,
                            alphabet,
                            character,
                            window_width,
                        );

                        ui.separator();
                        show_description(ui, space_node_description());
                    }
                }
            }

            if !problems.is_empty() {
                ui.separator();
                ui.label("Problems:");

                for problem in problems {
                    match problem {
                        NodeProblem::NotAvailable => {
                            ui.label(
                                "Node is not connected to any other node",
                            );
                        }
                        NodeProblem::StartNodeIncoming => {
                            ui.label(format!(
                                "Start node can't have incoming transitions"
                            ));
                        }
                        NodeProblem::EndNodeOutgoing => {
                            ui.label(format!(
                                "End node cant' have outgoing transitions"
                            ));
                        }
                        NodeProblem::MissingTransition(character) => {
                            ui.label(format!(
                                "Missing transition for '{character}'"
                            ));
                        }
                        NodeProblem::AmbiguousTransition(character) => {
                            ui.label(format!(
                                "Ambiguous transition for '{character}'"
                            ));
                        }
                    }
                }
            }
        });
}

fn choose_power(
    node: &mut Node,
    node_key: NodeKey,
    ui: &mut egui::Ui,
    power: u32,
    window_width: f32,
) {
    let state_id = Id::new(("node_power_edit", node_key));

    let mut state = ui.ctx().data_mut(|data| {
        data.get_temp::<PowerEditState>(state_id)
            .unwrap_or_else(|| PowerEditState {
                value: power.to_string(),
                select_all: true,
            })
    });

    ui.label("Power:");

    let response = ui.add(
        TextEdit::singleline(&mut state.value)
            .desired_width(window_width),
    );

    if state.select_all {
        select_all(ui.ctx(), response.id, state.value.chars().count());
        state.select_all = false;
    }

    if response.changed() {
        state.value.retain(|c| c.is_ascii_digit());

        let new_power = state
            .value
            .parse::<u32>()
            .unwrap_or(1)
            .max(1);

        match &mut node.action {
            Action::Left(power) | Action::Right(power) => {*power = new_power;},
            _ => {}
        };
    }

    ui.ctx().data_mut(|data| {
        data.insert_temp(state_id, state);
    });
}

fn choose_sign(
    node: &mut Node,
    node_key: NodeKey,
    ui: &mut egui::Ui,
    alphabet: &ValidatedAlphabet,
    current_char: char,
    window_width: f32,
) {
    // TODO add validation to sign
    let state_id = Id::new(("node_sign_edit", node_key));

    let mut state = ui.ctx().data_mut(|data| {
        data.get_temp::<SignEditState>(state_id)
            .unwrap_or_else(|| SignEditState {
                value: current_char.to_string(),
                select_all: true,
            })
    });

    state.value = formatted_buffer(
        &alphabet.iter().map(|c| *c).collect(),
        &state.value
    );

    ui.label("Sign:");

    let response = ui.add(
        TextEdit::singleline(&mut state.value)
            .desired_width(window_width),
    );

    if state.select_all {
        select_all(ui.ctx(), response.id, state.value.chars().count());
        state.select_all = false;
    }

    if response.changed() {
        let new_char = buffer_to_load(&state.value)
            .chars()
            .next()
            .unwrap_or(' ');

        node.action = Action::Write(new_char);
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

fn show_description(ui: &mut egui::Ui, description: &'static str) {
    ui.label(description);
}
