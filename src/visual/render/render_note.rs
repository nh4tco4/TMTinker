use egui::{Color32, Painter, Shape, epaint::RectShape};

use crate::{
    editor::{layout::GraphLayout, session::ViewportState}, visual::mappings::{
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
    painter: &Painter,
) {
    for (_id, note) in graph.notes_iter() {
        let screen_position: egui::Pos2 = viewport.world_to_screen(note.position).into();

        // let meta = if editor.selected.is_selected(&SelectableId::Note(*id)) {
        //     NoteMeta::Selected
        // } else if editor.selected.hovered == Some(SelectableId::Note(*id)) {
        //     NoteMeta::Hovered
        // } else {
        //     NoteMeta::Idle
        // };

        let meta = NoteMeta::Idle;

        let font_id = egui::FontId::proportional(NOTE_FONT_SIZE * viewport.camera_zoom.get());

        let text_galley =
            painter.layout_no_wrap(note.content.clone(), font_id.clone(), Color32::BLACK);
        let text_size = text_galley.size();

        let bg_size = egui::vec2(
            text_size.x + NOTE_PADDING * 2.0 * viewport.camera_zoom.get(),
            text_size.y + NOTE_PADDING * 2.0 * viewport.camera_zoom.get(),
        );

        painter.add(note_shape(screen_position, bg_size, meta, viewport.camera_zoom.get()));

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

fn note_shape(screen_pos: egui::Pos2, size: egui::Vec2, meta: NoteMeta, _zoom: f32) -> Shape {
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
