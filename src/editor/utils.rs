use crate::editor::layout::LayoutPos;

pub fn snap_position_to_grid(position: LayoutPos, grid_size: f32) -> LayoutPos {
    LayoutPos {
        x: (position.x / grid_size).round() * grid_size,
        y: (position.y / grid_size).round() * grid_size,
    }
}
