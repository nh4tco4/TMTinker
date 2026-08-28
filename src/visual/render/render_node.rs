use egui::{Color32, Painter, Rect, Shape, Vec2, epaint::RectShape};

use crate::{
    core::tm::{Action, NodeKey, Submachine}, editor::{layout::GraphLayout, session::{SelectableKey, SelectionState, ViewportState}}, visual::mappings::{
        BUTTON_CORNER_RADIUS, ICON_FONT_SIZE, IDLE_NODE_FILL, IDLE_NODE_STROKE, NODE_END, NODE_LEFT, NODE_RIGHT, NODE_SIZE, NODE_SPACE, NODE_START, NODE_STROKE_WIDTH,  PROBLEM_NODE_FILL, PROBLEM_NODE_STROKE, SELECTED_NODE_FILL, SELECTED_NODE_STROKE, SPACE,
    },
};

enum NodeMeta {
    Idle,
    Problem,
    Selected,
    Hovered,
}

pub fn render_nodes(
    graph: &GraphLayout,
    machine: &Submachine,
    viewport: &ViewportState,
    selection_state: &SelectionState,
    painter: &Painter,
) {
    let hovered_key: Option<NodeKey> = match selection_state.hovered {
        Some(key) => match key {
            SelectableKey::Node(node_key) => Some(node_key),
            _ => None,
        }
        None => None
    };

    for (key, node) in machine.nodes_iter() {
        let screen_position = viewport.world_to_screen(
            graph.get_node_position(key).expect("Unreachable")
        );

        let text_color = Color32::WHITE;

        let text_position = Rect::from_center_size(screen_position.into(), Vec2::splat(NODE_SIZE)).center();

        let is_problematic = !machine.get_node_problems(key).is_empty();
        let is_hovered = Some(key) == hovered_key;
        let is_selected = selection_state.selected_node_keys().contains(&key);

        let node_meta =  {
            if is_problematic {
                NodeMeta::Problem
            } else if is_selected {
                NodeMeta::Selected
            } else if is_hovered {
                NodeMeta::Hovered
            } else {
                NodeMeta::Idle
            }
            
        };

        painter.add(node_shape(screen_position.into(), node_meta, viewport.camera_zoom.get()));

        painter.text(
            text_position.into(),
            egui::Align2::CENTER_CENTER,
            match_node_label(&node.action),
            egui::FontId::proportional(ICON_FONT_SIZE * viewport.camera_zoom.get()),
            text_color,
        );

        match node.action.clone() {
            Action::Left(power)
            | Action::Right(power) => {
                if power > 1 {
                    painter.text(
                        text_position + Vec2::new(NODE_SIZE / 3.0, - NODE_SIZE / 3.0) * viewport.camera_zoom.get(),
                        egui::Align2::CENTER_CENTER,
                        power,
                        egui::FontId::proportional(ICON_FONT_SIZE / 1.5 * viewport.camera_zoom.get()),
                        egui::Color32::BLACK,
                    );
                }
            }
            Action::Submachine { power, name, .. } => {
                if power > 1 {
                    painter.text(
                        text_position + Vec2::new(NODE_SIZE / 3.0, - NODE_SIZE / 3.0) * viewport.camera_zoom.get(),
                        egui::Align2::CENTER_CENTER,
                        power,
                        egui::FontId::proportional(ICON_FONT_SIZE / 1.5 * viewport.camera_zoom.get()),
                        egui::Color32::BLACK,
                    );
                }

                painter.text(
                    text_position + Vec2::new(0.0, NODE_SIZE / 1.5) * viewport.camera_zoom.get(),
                    egui::Align2::CENTER_CENTER,
                    name,
                    egui::FontId::proportional(ICON_FONT_SIZE / 1.5 * viewport.camera_zoom.get()),
                    egui::Color32::BLACK,
                );
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
    };

    let size = NODE_SIZE * zoom;
    let rect =  Rect::from_center_size(screen_pos, Vec2::splat(size));

    Shape::Rect(RectShape::new(
        rect,
        BUTTON_CORNER_RADIUS,
        fill,
        egui::Stroke::new(NODE_STROKE_WIDTH, stroke_color),
        egui::StrokeKind::Middle,
    ))
}

fn match_node_label(action: &Action) -> String {
    match action {
        Action::Start => NODE_START.to_string(),
        Action::Stop => NODE_END.to_string(),
        Action::Left(_) => NODE_LEFT.to_string(),
        Action::Right(_) => NODE_RIGHT.to_string(),
        Action::Write(c) => {
            if *c == SPACE {
                NODE_SPACE.to_string()
            } else {
                c.to_string()
            }
        }
        Action::Submachine {
            name, ..
        } => name.chars().next().expect("Name can't be empty").to_string()
    }
}
