use egui::{Painter, Pos2};

use crate::{
    core::tm::{Submachine, TransitionSymbols},
    editor::{
        layout::GraphLayout,
        session::{SelectableKey, SelectionState, ViewportState},
    },
    visual::mappings::{
        BUTTON_SIZE,
        ICON_FONT_SIZE,
        IDLE_NODE_STROKE,
        NODE_SPACE,
        REGULAR_NODE_STROKE,
        SELECTED_NODE_STROKE,
    },
};

enum EdgeMeta {
    Idle,
    ConnectedToSelection,
    Selected,
    Hovered,
}

pub fn render_edges(
    graph: &GraphLayout,
    machine: &Submachine,
    viewport: &ViewportState,
    selection_state: &SelectionState,
    painter: &Painter,
) {
    let hovered_key = match selection_state.hovered {
        Some(SelectableKey::Edge(edge_key)) => Some(edge_key),
        _ => None,
    };

    let selected_nodes = selection_state.selected_node_keys();

    for (key, edge) in machine.edges_iter() {
        let source_position = viewport.world_to_screen(
            graph
                .get_node_position(edge.source)
                .expect("Unreachable. Edge can't exist without source."),
        );

        let target_position = viewport.world_to_screen(
            graph
                .get_node_position(edge.target)
                .expect("Unreachable. Edge can't exist without target."),
        );

        let is_loop = edge.source == edge.target;

        let points = build_edge_points(
            source_position.into(),
            target_position.into(),
            is_loop,
            viewport.camera_zoom.get(),
        );

        let is_hovered = Some(key) == hovered_key;

        let is_selected = selection_state
            .is_selected(&SelectableKey::Edge(key));

        let connects_selected_nodes =
            selected_nodes.contains(&edge.source)
                && selected_nodes.contains(&edge.target);

        let edge_meta = if is_selected {
            EdgeMeta::Selected
        } else if is_hovered {
            EdgeMeta::Hovered
        } else if connects_selected_nodes {
            EdgeMeta::ConnectedToSelection
        } else {
            EdgeMeta::Idle
        };

        let (stroke, color) = edge_style(edge_meta);

        draw_path(painter, &points, stroke);

        painter.add(arrow_tip(
            &points,
            color,
            viewport.camera_zoom.get(),
        ));

        let dot_r = 5.0 * viewport.camera_zoom.get();

        painter.circle_filled(
            *points.first().unwrap(),
            dot_r,
            color,
        );

        painter.circle_filled(
            *points.last().unwrap(),
            dot_r,
            color,
        );

        draw_edge_label(
            painter,
            &points,
            &edge.chars,
            viewport.camera_zoom.get(),
            color,
        );
    }
}

fn edge_style(meta: EdgeMeta) -> (egui::Stroke, egui::Color32) {
    match meta {
        EdgeMeta::Idle => (
            egui::Stroke::new(2.0, REGULAR_NODE_STROKE),
            REGULAR_NODE_STROKE,
        ),

        EdgeMeta::ConnectedToSelection => (
            egui::Stroke::new(2.5, SELECTED_NODE_STROKE),
            SELECTED_NODE_STROKE,
        ),

        EdgeMeta::Hovered => (
            egui::Stroke::new(2.5, SELECTED_NODE_STROKE),
            SELECTED_NODE_STROKE,
        ),

        EdgeMeta::Selected => (
            egui::Stroke::new(3.0, SELECTED_NODE_STROKE),
            SELECTED_NODE_STROKE,
        ),
    }
}

pub fn build_edge_points(
    src: Pos2,
    tgt: Pos2,
    is_loop: bool,
    zoom: f32,
) -> Vec<Pos2> {
    let node_size = BUTTON_SIZE * zoom;

    let src_mid_y = src.y;
    let tgt_mid_y = tgt.y;

    let start = Pos2::new(src.x + node_size / 2.0, src_mid_y);
    let end = Pos2::new(tgt.x - node_size / 2.0, tgt_mid_y);

    let mut points = vec![start];

    if is_loop {
        let ox = node_size;
        let oy = node_size * 2.0;

        points.extend([
            Pos2::new(start.x + ox, start.y),
            Pos2::new(start.x + ox, start.y + oy),
            Pos2::new(start.x - ox * 2.0, start.y + oy),
            Pos2::new(start.x - ox * 2.0, start.y),
            Pos2::new(start.x - ox, start.y),
        ]);
    } else if src.x + 100.0 < tgt.x {
        let mid_x = f32::midpoint(start.x, end.x);

        points.extend([
            Pos2::new(mid_x, src_mid_y),
            Pos2::new(mid_x, tgt_mid_y),
            end,
        ]);
    } else {
        let offset = node_size;
        let mid_y = f32::midpoint(src_mid_y, tgt_mid_y);

        points.extend([
            Pos2::new(src.x + offset, src_mid_y),
            Pos2::new(src.x + offset, mid_y),
            Pos2::new(tgt.x - offset, mid_y),
            Pos2::new(tgt.x - offset, tgt_mid_y),
            end,
        ]);
    }

    points
}

fn draw_path(
    painter: &Painter,
    points: &[Pos2],
    stroke: egui::Stroke,
) {
    for window in points.windows(2) {
        painter.line_segment(
            [window[0], window[1]],
            stroke,
        );
    }
}

fn arrow_tip(
    points: &[Pos2],
    color: egui::Color32,
    zoom: f32,
) -> egui::Shape {
    let tip = *points
        .last()
        .expect("arrow_tip called with empty points");

    let prev = points[points.len() - 2];

    let arrow_width = 6.0 * zoom;
    let arrow_len = 10.0 * zoom;

    let dir = (tip - prev).normalized();
    let perp = egui::Vec2::new(-dir.y, dir.x) * arrow_width;
    let base = tip - dir * (arrow_len + 5.0);

    egui::Shape::convex_polygon(
        vec![
            tip,
            base + perp,
            base - perp,
        ],
        color,
        egui::Stroke::NONE,
    )
}

fn draw_edge_label(
    painter: &Painter,
    points: &[Pos2],
    chars: &TransitionSymbols,
    zoom: f32,
    color: egui::Color32,
) {
    let tip = *points
        .last()
        .expect("draw_edge_label called with empty points");

    let prev = points[points.len() - 2];
    let dir = (tip - prev).normalized();

    let arrow_len = 10.0 * zoom;
    let label_distance = 25.0 * zoom;
    let label_offset =
        egui::Vec2::new(0.0, -20.0 * zoom);

    let label_pos =
        tip
            - dir * (arrow_len + label_distance)
            + label_offset;

    let display: String = chars
        .iter()
        .map(|c| {
            if c == ' ' {
                NODE_SPACE.to_owned()
            } else {
                c.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("");

    if !display.is_empty() {
        let font_zoom = zoom.clamp(0.7, 1.5);

        painter.text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            display,
            egui::FontId::proportional(
                ICON_FONT_SIZE * font_zoom,
            ),
            color,
        );
    }
}
