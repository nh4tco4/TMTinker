use egui::{Color32, Painter, Rect, Shape, Vec2, epaint::RectShape};

use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::compile::diagnostics::node_problems,
    diagram::editor::session::{SelectableKey, SelectionState, ViewportState},
    diagram::model::layout::GraphLayout,
    diagram::model::{
        node::{Direction, NodeAction},
        submachine::{NodeKey, Submachine},
    },
    ui::theme::{
        ACTIVE_NODE_FILL, ACTIVE_NODE_STROKE, BUTTON_CORNER_RADIUS, ICON_FONT_SIZE, IDLE_NODE_FILL,
        IDLE_NODE_STROKE, NODE_END, NODE_LEFT, NODE_RIGHT, NODE_SIZE, NODE_SPACE, NODE_START,
        NODE_STROKE_WIDTH, PROBLEM_NODE_FILL, PROBLEM_NODE_STROKE, SELECTED_NODE_FILL,
        SELECTED_NODE_STROKE, SPACE,
    },
};

enum NodeMeta {
    Idle,
    Problem,
    Selected,
    Hovered,
    Executing,
}

pub fn render_nodes(
    graph: &GraphLayout,
    machine: &Submachine,
    alphabet: &ValidatedAlphabet,
    viewport: &ViewportState,
    selection_state: &SelectionState,
    executing_node: Option<NodeKey>,
    painter: &Painter,
) {
    let hovered_key: Option<NodeKey> = match selection_state.hovered {
        Some(key) => match key {
            SelectableKey::Node(node_key) => Some(node_key),
            _ => None,
        },
        None => None,
    };

    for (key, node) in machine.nodes_iter() {
        let screen_position =
            viewport.world_to_screen(graph.get_node_position(key).expect("Unreachable"));

        let text_color = Color32::BLACK;

        let text_position =
            Rect::from_center_size(screen_position.into(), Vec2::splat(NODE_SIZE)).center();

        let is_problematic = !node_problems(machine, alphabet, key).is_empty();
        let is_hovered = Some(key) == hovered_key;
        let is_selected = selection_state.selected_node_keys().contains(&key);

        let node_meta = {
            if executing_node == Some(key) {
                NodeMeta::Executing
            } else if is_selected {
                NodeMeta::Selected
            } else if is_hovered {
                NodeMeta::Hovered
            } else if is_problematic {
                NodeMeta::Problem
            } else {
                NodeMeta::Idle
            }
        };

        painter.add(node_shape(
            screen_position.into(),
            node_meta,
            viewport.camera_zoom.get(),
        ));

        painter.text(
            text_position.into(),
            egui::Align2::CENTER_CENTER,
            match_node_label(&node.action()),
            egui::FontId::proportional(ICON_FONT_SIZE * viewport.camera_zoom.get()),
            text_color,
        );

        match node.action().clone() {
            NodeAction::Move { repeat, .. } => {
                if repeat.get() > 1 {
                    painter.text(
                        text_position
                            + Vec2::new(NODE_SIZE / 3.0, -NODE_SIZE / 3.0)
                                * viewport.camera_zoom.get(),
                        egui::Align2::CENTER_CENTER,
                        repeat.get(),
                        egui::FontId::proportional(
                            ICON_FONT_SIZE / 1.5 * viewport.camera_zoom.get(),
                        ),
                        egui::Color32::BLACK,
                    );
                }
            }

            NodeAction::Call { repeat, .. } => {
                if repeat.get() > 1 {
                    painter.text(
                        text_position
                            + Vec2::new(NODE_SIZE / 3.0, -NODE_SIZE / 3.0)
                                * viewport.camera_zoom.get(),
                        egui::Align2::CENTER_CENTER,
                        repeat.get(),
                        egui::FontId::proportional(
                            ICON_FONT_SIZE / 1.5 * viewport.camera_zoom.get(),
                        ),
                        egui::Color32::BLACK,
                    );
                }
            }
            _ => {}
        }
    }
}

fn node_shape(screen_pos: egui::Pos2, meta: NodeMeta, zoom: f32) -> Shape {
    let (stroke_color, fill) = match meta {
        NodeMeta::Idle => (IDLE_NODE_STROKE, IDLE_NODE_FILL),
        NodeMeta::Problem => (PROBLEM_NODE_STROKE, PROBLEM_NODE_FILL),
        NodeMeta::Selected => (SELECTED_NODE_STROKE, SELECTED_NODE_FILL),
        NodeMeta::Hovered => (SELECTED_NODE_STROKE, IDLE_NODE_FILL),
        NodeMeta::Executing => (ACTIVE_NODE_STROKE, ACTIVE_NODE_FILL),
    };

    let size = NODE_SIZE * zoom;
    let rect = Rect::from_center_size(screen_pos, Vec2::splat(size));

    Shape::Rect(RectShape::new(
        rect,
        BUTTON_CORNER_RADIUS,
        fill,
        egui::Stroke::new(NODE_STROKE_WIDTH * zoom, stroke_color),
        egui::StrokeKind::Middle,
    ))
}

fn match_node_label(action: &NodeAction) -> String {
    match action {
        NodeAction::Start => NODE_START.to_string(),
        NodeAction::Stop => NODE_END.to_string(),
        NodeAction::Move { direction, .. } => match direction {
            Direction::Left => NODE_LEFT.to_string(),
            Direction::Right => NODE_RIGHT.to_string(),
        },
        NodeAction::Write(c) => {
            if *c == SPACE {
                NODE_SPACE.to_string()
            } else {
                c.to_string()
            }
        }
        NodeAction::Call { .. } => "S".to_string(),
    }
}
