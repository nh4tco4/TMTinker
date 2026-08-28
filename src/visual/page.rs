// use egui::scroll_area::ScrollBarVisibility;
//
// use crate::{
//     app::App, core::graph::TransitionSymbols, visual::{
//         canvas::handle_canvas,
//         mappings::{BUTTON_SIZE, CANVAS_BACKGROUND},
//         toolbars::{
//             Page, control_window::control, node_control_windows::node_options_base,
//             tape_window::tape, tools_window::tools,
//         },
//         utils::text_input::{buffer_to_load, formatted_buffer},
//     },
// };
//
// const TAPE_CELL_SPACING: f32 = 4.0;
// const TAPE_BUTTON_WIDTH: f32 = BUTTON_SIZE + 2.0 * TAPE_CELL_SPACING;
// const CONTROL_BUTTONS_WIDTH: f32 = TAPE_CELL_SPACING * TAPE_BUTTON_WIDTH;
//
// /// Renders the tinkering page: canvas + all floating panels.
// pub fn render_tinker(app: &mut App, ctx: &egui::Context) {
//     // // --- Tape window
//     // let usable_width = ctx.content_rect().width() - 20.0;
//     // let tape_area_width = usable_width - CONTROL_BUTTONS_WIDTH;
//     // let num_half_cells = ((tape_area_width / TAPE_BUTTON_WIDTH) / 2.0).floor() as i32;
//     // let total_tape_width = (2 * num_half_cells) as f32 * TAPE_BUTTON_WIDTH + CONTROL_BUTTONS_WIDTH;
//     // let tape_padding = ((ctx.content_rect().width() - total_tape_width) / 2.0).max(0.0);
//
//     // egui::Window::new("Tape")
//     //     .title_bar(true)
//     //     .resizable(false)
//     //     .movable(true)
//     //     .fixed_pos(egui::pos2(tape_padding, 20.0))
//     //     .show(ctx, |ui| {
//     //         tape(app, ui, num_half_cells.max(1));
//     //     });
//
// }
//
//
