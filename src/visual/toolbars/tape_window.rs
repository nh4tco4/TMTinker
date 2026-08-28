// use crate::{
//     app::App,
//     visual::{
//         mappings::{
//             ACTIVE_NODE_STROKE, BUTTON_CORNER_RADIUS, BUTTON_SIZE, BUTTON_STROKE_WIDTH,
//             ICON_FONT_SIZE, LAMBDA, NODE_SPACE, SELECTED_NODE_STROKE, SPACE, SUBSCRIPT_FONT_SIZE,
//             WORKING_NODE_STROKE,
//         },
//         utils::text_input::{buffer_to_load, formatted_buffer},
//     },
// };
// use egui::{Align, Id, Layout, Response, RichText, TextEdit, Ui};
//
// pub fn tape(app: &mut App, ui: &mut Ui, num_half_cells: i32) {
//     ui.horizontal(|ui| {
//         move_button(ui, app, 'L');
//         move_button(ui, app, '<');
//
//         for i in -num_half_cells + app.tape_offset..num_half_cells + app.tape_offset {
//             tape_button(ui, app, i);
//         }
//
//         move_button(ui, app, '>');
//         move_button(ui, app, 'R');
//     });
// }
//
// fn tape_button(ui: &mut Ui, app: &mut App, cell_id: i32) {
//     let space_display = NODE_SPACE.chars().next().unwrap_or('λ');
//     let cell_char = app
//         .tm
//         .tm_impl
//         .tape_items()
//         .get(&cell_id)
//         .copied()
//         .unwrap_or(' ');
//     let display_char = if cell_char == ' ' {
//         space_display
//     } else {
//         cell_char
//     };
//
//     let button_size = egui::vec2(BUTTON_SIZE, BUTTON_SIZE);
//     let is_head = app.tm.tm_impl.head_position() == cell_id;
//     let is_editing = app.edit_cell == Some(cell_id);
//
//     ui.allocate_ui_with_layout(
//         egui::vec2(button_size.x, button_size.y + 20.0),
//         Layout::top_down(Align::Center),
//         |ui| {
//             if is_editing {
//                 draw_edit_cell(ui, app, cell_id, button_size);
//             } else {
//                 draw_normal_cell(ui, app, cell_id, display_char, button_size, is_head);
//             }
//         },
//     );
// }
//
// fn draw_normal_cell(
//     ui: &mut Ui,
//     app: &mut App,
//     cell_id: i32,
//     display_char: char,
//     button_size: egui::Vec2,
//     is_head: bool,
// ) {
//     let text_color = if is_head {
//         if !app.is_halted {
//             WORKING_NODE_STROKE
//         } else {
//             SELECTED_NODE_STROKE
//         }
//     } else {
//         ui.visuals().text_color()
//     };
//
//     let response = ui.add_sized(
//         button_size,
//         egui::Button::new(
//             RichText::new(display_char.to_string())
//                 .size(ICON_FONT_SIZE)
//                 .color(text_color),
//         )
//         .frame(true),
//     );
//
//     if response.clicked() && !app.test_mode {
//         app.tm.tm_impl.seek(cell_id);
//     }
//
//     if response.secondary_clicked() && !app.test_mode {
//         app.edit_cell = Some(cell_id);
//         app.tm.tm_impl.seek(cell_id);
//
//         app.edit_cell_buffer = String::new();
//
//         ui.memory_mut(|m| m.request_focus(Id::new("tape_text_edit").with(cell_id)));
//     }
//
//     if is_head {
//         draw_selected_head(app, ui, &response);
//     }
// }
//
// fn draw_edit_cell(ui: &mut Ui, app: &mut App, cell_id: i32, button_size: egui::Vec2) {
//     let edit_id = Id::new("tape_text_edit").with(cell_id);
//
//     app.edit_cell_buffer.truncate(1);
//     app.edit_cell_buffer = formatted_buffer(&app.alphabet, &app.edit_cell_buffer);
//
//     let text_buf = &mut app.edit_cell_buffer;
//
//     let edit_response = ui
//         .allocate_ui_with_layout(
//             button_size,
//             Layout::centered_and_justified(egui::Direction::TopDown),
//             |ui| {
//                 ui.add_sized(
//                     button_size,
//                     TextEdit::singleline(text_buf)
//                         .id(edit_id)
//                         .desired_width(button_size.x - 8.0)
//                         .font(egui::FontId::proportional(ICON_FONT_SIZE))
//                         .frame(false)
//                         .vertical_align(Align::Center)
//                         .horizontal_align(Align::Center),
//                 )
//             },
//         )
//         .inner;
//
//     let mut should_close = edit_response.lost_focus();
//     let mut commit_char: Option<char> = None;
//
//     ui.input(|i| {
//         if i.key_pressed(egui::Key::Enter) {
//             should_close = true;
//             commit_char = buffer_to_load(text_buf).chars().next();
//         } else if i.key_pressed(egui::Key::Escape) {
//             should_close = true;
//         }
//     });
//
//     if edit_response.changed() {
//         should_close = true;
//         commit_char = buffer_to_load(text_buf).chars().next();
//     }
//
//     if should_close {
//         if let Some(ch) = commit_char {
//             let new_ch = if ch == LAMBDA { SPACE } else { ch };
//             app.tm.tm_impl.write_at(cell_id, new_ch);
//         }
//         app.edit_cell = None;
//         app.edit_cell_buffer.clear();
//     }
//
//     if app.edit_cell.is_some() {
//         draw_edit_head(ui, &edit_response);
//     }
// }
//
// fn move_button(ui: &mut Ui, app: &mut App, direction: char) {
//     let button_size = egui::vec2(BUTTON_SIZE, BUTTON_SIZE);
//     let text = match direction {
//         '>' => ">",
//         '<' => "<",
//         'L' => "<<",
//         'R' => ">>",
//         _ => "?",
//     };
//
//     ui.allocate_ui_with_layout(
//         egui::vec2(button_size.x, button_size.y + 20.0),
//         Layout::top_down(Align::Center),
//         |ui| {
//             let response = ui.add_sized(
//                 button_size,
//                 egui::Button::new(
//                     RichText::new(text)
//                         .size(SUBSCRIPT_FONT_SIZE)
//                         .color(ui.visuals().text_color()),
//                 )
//                 .frame(true),
//             );
//
//             if response.clicked() {
//                 match direction {
//                     '>' => app.tape_offset -= 1,
//                     '<' => app.tape_offset += 1,
//                     'R' => app.tape_offset -= 5,
//                     'L' => app.tape_offset += 5,
//                     _ => {}
//                 }
//             }
//         },
//     );
// }
//
// fn draw_selected_head(app: &mut App, ui: &mut Ui, response: &Response) {
//     let rect = response.rect.expand2(egui::vec2(2.0, 2.0));
//     ui.painter().rect_stroke(
//         rect,
//         BUTTON_CORNER_RADIUS,
//         egui::Stroke::new(
//             BUTTON_STROKE_WIDTH,
//             if !app.is_halted {
//                 WORKING_NODE_STROKE
//             } else {
//                 SELECTED_NODE_STROKE
//             },
//         ),
//         egui::StrokeKind::Inside,
//     );
// }
//
// fn draw_edit_head(ui: &mut Ui, response: &Response) {
//     let rect = response.rect.expand2(egui::vec2(3.0, 3.0));
//     ui.painter().rect_stroke(
//         rect,
//         BUTTON_CORNER_RADIUS,
//         egui::Stroke::new(BUTTON_STROKE_WIDTH * 1.5, ACTIVE_NODE_STROKE),
//         egui::StrokeKind::Inside,
//     );
// }
