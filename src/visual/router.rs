use crate::{project::Project, visual::{canvas::handle_canvas, toolbars::{control_window::{ControlAction, control_window}, tools_window::select_tool}, tools::dyn_tools_base::EditorTool}};

#[derive(Clone, Default)]
pub struct ToolState {
    pub current_tool: EditorTool,
}

#[derive(Clone, Default)]
pub struct TapeState {
    pub current_cell_id: u32,
}

pub enum Page {
    Tinkering,
    Menu,
}

pub enum TinkerAction {
    None,
    GoToMenu,
    SaveProject,
}

pub fn render_tinker(project: &mut Project, ctx: &egui::Context) -> TinkerAction {
    let mut tinker_action = TinkerAction::None;

    let id = egui::Id::new("tool_state");

    let mut tool_state = ctx.data(|d| {
        d.get_temp::<ToolState>(id).unwrap_or_default()
    });

    egui::Window::new("Tools")
        .title_bar(true)
        .resizable(false)
        .movable(true)
        .min_height(1000.0)
        .default_pos(egui::pos2(1200.0, 150.0))
        .scroll_bar_visibility(
            egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded
        )
        .show(ctx, |ui| {
            select_tool(project, ui, &mut tool_state, ctx);
        });

    egui::Window::new("Control")
        .title_bar(true)
        .resizable(false)
        .movable(true)
        .min_height(1000.0)
        .default_pos(egui::pos2(10.0, 150.0))
        .scroll_bar_visibility(
            egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded
        )
        .show(ctx, |ui| {
            match control_window(project, ui) {
                ControlAction::Menu => {
                    tinker_action = TinkerAction::GoToMenu;
                }
                ControlAction::Run => {
                    // TODO run the machine
                }
                ControlAction::Save => {
                    // TODO save project + auto_save
                }
                ControlAction::None
                | ControlAction::Help => (),
            };
        });

    let alphabet = project.get_alphabet().clone();
    egui::CentralPanel::default()
        .frame(egui::Frame::default())
        .show(ctx, |ui| {
            handle_canvas(
                project.current_document_mut(),
                &tool_state.current_tool,
                alphabet,
                ctx,
                ui,
            );
        });

    ctx.data_mut(|d| {
        d.insert_temp(id, tool_state);
    });

    tinker_action
}

// pub fn render_tinker(project: &mut Project, ui: &mut Ui, ctx: &Context) {
//     let id = ui.id().with("tool_state");
//     let mut tool_state = ctx.data(|d| {
//         d.get_temp::<ToolState>(id).unwrap_or_default()
//     });
//
//
//
//     // --- Canvas (central panel)
//     egui::CentralPanel::default()
//         .frame(egui::Frame::default())
//         .show(ctx, |ui| {
//             handle_canvas(
//                 project.current_document_mut(),
//                 &tool_state.current_tool,
//                 ctx,
//                 ui,
//             );
//         });
//
//     // // --- Tape window
//     // let usable_width = ctx.content_rect().width() - 20.0;
//     // let tape_area_width = usable_width - CONTROL_BUTTONS_WIDTH;
//     // let num_half_cells = ((tape_area_width / TAPE_BUTTON_WIDTH) / 2.0).floor() as i32;
//     // let total_tape_width = (2 * num_half_cells) as f32 * TAPE_BUTTON_WIDTH + CONTROL_BUTTONS_WIDTH;
//     // let tape_padding = ((ctx.content_rect().width() - total_tape_width) / 2.0).max(0.0);
//     //
//     // egui::Window::new("Tape")
//     //     .title_bar(true)
//     //     .resizable(false)
//     //     .movable(true)
//     //     .fixed_pos(egui::pos2(tape_padding, 20.0))
//     //     .show(ctx, |ui| {
//     //         tape(app, ui, num_half_cells.max(1));
//     //     });
//
//     // --- Tools window
//     egui::Window::new("Tools")
//         .title_bar(true)
//         .resizable(false)
//         .movable(true)
//         .min_height(1000.0)
//         .default_pos(egui::pos2(1200.0, 150.0))
//         .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
//         .show(ctx, |ui| {
//             select_tool(project, ui, &mut tool_state);
//         });
//     control_window(project, ui);
//
//
//
//     ctx.data_mut(|d| {
//         d.insert_temp(id, tool_state);
//     });
// }

// pub fn render_debugger(project: &mut Project, ui: &mut Ui, ctx: &Context) {
//     let id = ui.id().with("tape_state");
//
//     let mut tool_state = ctx.data(|d| {
//         d.get_temp::<ToolState>(id).unwrap_or_default()
//     });
//
//     select_tool(project, ui, &mut tool_state);
//
//     handle_canvas(
//         project.current_document_mut(),
//         &tool_state.current_tool,
//         ctx,
//         ui,
//     );
//
//     ctx.data_mut(|d| {
//         d.insert_temp(id, tool_state);
//     });
// }

// fn handle_hotkeys(app: &mut App, ctx: &egui::Context) {
//     use egui::{Key, Modifiers};
//
//     let mut action_taken = false;
//
//     ctx.input_mut(|i| {
//         if i.consume_key(Modifiers::CTRL, Key::Z) {
//             UndoManager::undo(app);
//             action_taken = true;
//         }
//         if i.consume_key(Modifiers::CTRL, Key::Y) {
//             UndoManager::redo(app);
//             action_taken = true;
//         }
//
//         // TODO! probably make it into a proc macro
//         if app.current_editor().selected.is_empty() {
//             if i.consume_key(Modifiers::NONE, move_node_shortcut()) {
//                 app.current_tool = EditorTool::Edit(EditTools::Move);
//             } else if i.consume_key(Modifiers::NONE, delete_node_shortcut()) {
//                 app.current_tool = EditorTool::Edit(EditTools::Delete);
//             } else if i.consume_key(Modifiers::NONE, link_node_shortcut()) {
//                 app.current_tool = EditorTool::Edit(EditTools::Link);
//                 app.link_source = None;
//             } else if i.consume_key(Modifiers::NONE, note_node_shorcut()) {
//                 app.current_tool = EditorTool::Edit(EditTools::Comment);
//             } else if i.consume_key(Modifiers::NONE, left_node_shortcut()) {
//                 app.current_tool = EditorTool::Place(BasicActionTools::Left);
//             } else if i.consume_key(Modifiers::NONE, right_node_shortcut()) {
//                 app.current_tool = EditorTool::Place(BasicActionTools::Right);
//             } else if i.consume_key(Modifiers::NONE, space_node_shortcut()) {
//                 app.current_tool = EditorTool::Place(BasicActionTools::Write);
//             } else if i.consume_key(Modifiers::NONE, end_node_shortcut()) {
//                 app.current_tool = EditorTool::Place(BasicActionTools::Stop);
//             } else if i.consume_key(Modifiers::NONE, start_node_shortcut()) {
//                 app.current_tool = EditorTool::Place(BasicActionTools::Start);
//             } else if i.consume_key(Modifiers::NONE, Key::ArrowRight) {
//                 let p = app.tm.tm_impl.head_position() + 1;
//                 app.tm.tm_impl.seek(p);
//             } else if i.consume_key(Modifiers::NONE, Key::ArrowLeft) {
//                 let p = app.tm.tm_impl.head_position() - 1;
//                 app.tm.tm_impl.seek(p);
//             } else if i.consume_key(Modifiers::NONE, Key::Escape) {
//                 app.dragged_nodes.clear();
//                 app.current_tool = EditorTool::Edit(EditTools::Move);
//                 app.current_editor_mut().selected.clear();
//             }
//         } else {
//             if i.consume_key(Modifiers::NONE, Key::Escape) {
//                 app.current_editor_mut().selected.clear();
//                 app.current_tool = EditorTool::Edit(EditTools::Move);
//                 app.dragged_nodes.clear();
//                 app.editing_note = None;
//             } else if i.consume_key(Modifiers::NONE, Key::Delete) {
//                 let (graph, editor) = app.current_graph_and_editor_mut();
//                 for key in editor.selected.selected_node_keys() {
//                     graph.remove_node(key);
//                 }
//                 action_taken = true;
//             }
//         }
//
//         for event in &i.events {
//             if let egui::Event::Copy = event {
//                 copy_to_buf(app, ctx.clone());
//                 action_taken = true;
//             }
//             if let egui::Event::Paste(_) = event {
//                 if let Some(screen_pos) = i.pointer.latest_pos() {
//                     paste_copy_buf(app, screen_pos.into());
//                     action_taken = true;
//                 }
//             }
//         }
//     });
//
//     if action_taken {
//         UndoManager::save_checkpoint(app);
//     }
// }
