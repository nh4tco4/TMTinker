use egui::{Color32, Painter, Shape, epaint::RectShape};

use crate::{
    diagram::{
        editor::session::{SelectableKey, SelectionState, ViewportState},
        model::layout::{GraphLayout, Note},
    },
    ui::theme::{
        IDLE_NODE_FILL, IDLE_NODE_STROKE, NODE_STROKE_WIDTH, SELECTED_NODE_FILL,
        SELECTED_NODE_STROKE,
    },
};

const NOTE_CORNER_RADIUS: f32 = 8.0;
const NOTE_PADDING: f32 = 8.0;
const NOTE_FONT_SIZE: f32 = 14.0;

enum NoteMeta {
    Idle,
    Selected,
    Hovered,
}

pub fn render_notes(
    graph: &GraphLayout,
    viewport: &ViewportState,
    selection: &SelectionState,
    painter: &Painter,
) {
    for (id, note) in graph.notes_iter() {
        let key = SelectableKey::Note(id);
        let meta = if selection.is_selected(&key) {
            NoteMeta::Selected
        } else if selection.hovered == Some(key) {
            NoteMeta::Hovered
        } else {
            NoteMeta::Idle
        };
        let rect = note_rect(note, viewport, painter);
        let screen_position = rect.min;
        let bg_size = rect.size();

        painter.add(note_shape(screen_position, bg_size, meta));

        let text_pos = screen_position + egui::Vec2::new(bg_size.x / 2.0, bg_size.y / 2.0);
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            &note.content,
            egui::FontId::proportional(NOTE_FONT_SIZE * viewport.camera_zoom.get()),
            Color32::BLACK,
        );
    }
}

pub fn note_rect(note: &Note, viewport: &ViewportState, painter: &Painter) -> egui::Rect {
    let zoom = viewport.camera_zoom.get();
    let galley = painter.layout_no_wrap(
        note.content.clone(),
        egui::FontId::proportional(NOTE_FONT_SIZE * zoom),
        Color32::BLACK,
    );
    egui::Rect::from_center_size(
        viewport.world_to_screen(note.position).into(),
        galley.size() + egui::Vec2::splat(NOTE_PADDING * 2. * zoom),
    )
}

fn note_shape(screen_pos: egui::Pos2, size: egui::Vec2, meta: NoteMeta) -> Shape {
    let (stroke_color, fill) = match meta {
        NoteMeta::Idle => (IDLE_NODE_STROKE, IDLE_NODE_FILL),
        NoteMeta::Selected => (SELECTED_NODE_STROKE, SELECTED_NODE_FILL),
        NoteMeta::Hovered => (SELECTED_NODE_STROKE, IDLE_NODE_FILL),
    };

    let rect = egui::Rect::from_min_size(screen_pos, size);

    Shape::Rect(RectShape::new(
        rect,
        NOTE_CORNER_RADIUS,
        fill,
        egui::Stroke::new(NODE_STROKE_WIDTH, stroke_color),
        egui::StrokeKind::Middle,
    ))
}
