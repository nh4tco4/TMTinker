use egui::{Painter, Rect};

use crate::{
    editor::{layout::{GraphLayout, LayoutPos}, session::ViewportState}, visual::mappings::{CANVAS_BACKGROUND, CANVAS_GRID},
};

pub fn render_grid(viewport: &ViewportState, rect: Rect, painter: &Painter) {
    painter.rect_filled(rect, 0.0, CANVAS_BACKGROUND);

    let effective_spacing = GraphLayout::GRID_SIZE * viewport.camera_zoom.get();
    if effective_spacing <= 0.0 {
        return;
    }

    let world_origin_screen = viewport.world_to_screen(LayoutPos { x: 0.0, y: 0.0 });

    let offset_x = (world_origin_screen.x - rect.left()) % effective_spacing;
    let start_x = rect.left()
        + (if offset_x >= 0.0 {
            offset_x
        } else {
            effective_spacing + offset_x
        });

    let offset_y = (world_origin_screen.y - rect.top()) % effective_spacing;
    let start_y = rect.top()
        + (if offset_y >= 0.0 {
            offset_y
        } else {
            effective_spacing + offset_y
        });

    let mut x = start_x;
    while x <= rect.right() {
        let mut y = start_y;
        while y <= rect.bottom() {
            painter.circle_filled(egui::Pos2::new(x, y), 3.0 * viewport.camera_zoom.get(), CANVAS_GRID);
            y += effective_spacing;
        }
        x += effective_spacing;
    }
}
