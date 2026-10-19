use egui::{
    Id, TextEdit,
    text::{CCursor, CCursorRange},
};
use std::num::NonZeroU32;

use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::editor::session::ViewportState,
    diagram::model::layout::LayoutPos,
    diagram::model::node::{Direction, Node, NodeAction},
    diagram::ui::editor::descriptions::*,
    diagram::{
        compile::diagnostics::{NodeProblem, node_problems},
        model::submachine::{NodeKey, Submachine},
    },
    ui::{
        theme::NODE_SIZE,
        widgets::text_input::{buffer_to_load, formatted_buffer},
    },
};

#[derive(Clone)]
pub struct PowerEditState {
    value: String,
    select_all: bool,
}

#[derive(Clone)]
pub struct SignEditState {
    value: String,
    select_all: bool,
}

#[derive(Default)]
pub struct NodeInspectorState {
    powers: std::collections::HashMap<NodeKey, PowerEditState>,
    signs: std::collections::HashMap<NodeKey, SignEditState>,
}

pub fn node_options_base(
    viewport: &ViewportState,
    submachine: &mut Submachine,
    alphabet: &ValidatedAlphabet,
    ctx: &egui::Context,
    node_key: NodeKey,
    window_pos: egui::Pos2,
    state: &mut NodeInspectorState,
) {
    let shift = LayoutPos::new(NODE_SIZE, NODE_SIZE) * viewport.camera_zoom.get();
    let new_pos = viewport.screen_to_world(shift + window_pos.into());

    let problems = node_problems(submachine, alphabet, node_key);

    egui::Window::new("Node Options")
        .id(Id::new(("node_options", node_key)))
        .title_bar(true)
        .resizable(false)
        .fixed_pos(new_pos)
        .movable(true)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ctx, |ui| {
            if let Some(node) = submachine.get_mut_node(node_key) {
                let window_width = ui.available_width();

                let action = node.action().clone();

                match action {
                    NodeAction::Start => {
                        show_description(ui, START_NODE);
                    }
                    NodeAction::Stop => {
                        show_description(ui, END_NODE);
                    }
                    NodeAction::Move { direction, repeat } => {
                        choose_power(node, node_key, ui, repeat, window_width, state);

                        ui.separator();
                        match direction {
                            Direction::Left => show_description(ui, LEFT_NODE),
                            Direction::Right => show_description(ui, RIGHT_NODE),
                        }
                    }
                    NodeAction::Call { repeat, .. } => {
                        choose_power(node, node_key, ui, repeat, window_width, state);

                        ui.separator();
                        show_description(ui, SUBMACHINE_NODE);
                    }
                    NodeAction::Write(character) => {
                        choose_sign(node, node_key, ui, alphabet, character, window_width, state);

                        ui.separator();
                        show_description(ui, SPACE_NODE);
                    }
                }
            }

            if !problems.is_empty() {
                ui.separator();
                ui.label("Problems:");

                for problem in &problems {
                    match problem {
                        NodeProblem::NotAvailable => {
                            ui.label("Node is not connected to any other node");
                        }
                        NodeProblem::StartNodeIncoming => {
                            ui.label("Start node can't have incoming transitions");
                        }
                        NodeProblem::EndNodeOutgoing => {
                            ui.label("End node cant' have outgoing transitions");
                        }
                        NodeProblem::MissingTransition(character) => {
                            let character = if *character == ' ' { 'λ' } else { *character };
                            ui.label(format!("Missing transition for '{character}'"));
                        }
                        NodeProblem::AmbiguousTransition(character) => {
                            let character = if *character == ' ' { 'λ' } else { *character };
                            ui.label(format!("Ambiguous transition for '{character}'"));
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
    power: NonZeroU32,
    window_width: f32,
    inspector: &mut NodeInspectorState,
) {
    let state = inspector
        .powers
        .entry(node_key)
        .or_insert_with(|| PowerEditState {
            value: power.get().to_string(),
            select_all: true,
        });

    ui.label("Power:");

    let response = ui.add(TextEdit::singleline(&mut state.value).desired_width(window_width));

    if state.select_all {
        select_all(ui.ctx(), response.id, state.value.chars().count());
        state.select_all = false;
    }

    if response.changed() {
        state.value.retain(|c| c.is_ascii_digit());

        let new_power_raw = state.value.parse::<u32>().unwrap_or(1).max(1);

        let new_power = NonZeroU32::new(new_power_raw).unwrap_or(NonZeroU32::MIN);

        let mut new_action = None;

        match node.action() {
            NodeAction::Move { direction, .. } => {
                new_action = Some(NodeAction::Move {
                    repeat: new_power,
                    direction: direction.clone(),
                })
            }
            NodeAction::Call { target, .. } => {
                new_action = Some(NodeAction::Call {
                    repeat: new_power,
                    target: *target,
                })
            }
            _ => {}
        };

        if let Some(action) = new_action {
            node.set_action(action);
        }
    }
}

fn choose_sign(
    node: &mut Node,
    node_key: NodeKey,
    ui: &mut egui::Ui,
    alphabet: &ValidatedAlphabet,
    current_char: char,
    window_width: f32,
    inspector: &mut NodeInspectorState,
) {
    let state = inspector
        .signs
        .entry(node_key)
        .or_insert_with(|| SignEditState {
            value: current_char.to_string(),
            select_all: true,
        });

    state.value = formatted_buffer(alphabet, &state.value);

    ui.label("Sign:");

    let response = ui.add(TextEdit::singleline(&mut state.value).desired_width(window_width));

    if state.select_all {
        select_all(ui.ctx(), response.id, state.value.chars().count());
        state.select_all = false;
    }

    if response.changed() {
        state.value = formatted_buffer(alphabet, &state.value);
        let new_char = buffer_to_load(&state.value).chars().next().unwrap_or(' ');

        node.set_action(NodeAction::Write(new_char));
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

fn show_description(ui: &mut egui::Ui, description: &'static str) {
    ui.label(description);
}
