use super::{
    control_panel::control_panel,
    tools_panel::{dialogs, tools_panel},
};
use crate::{
    commands::Command,
    diagram::{
        editor::session::DiagramEditorSession, model::submachine::SubmachineKey,
        ui::canvas::editor::handle_canvas,
    },
    project::{
        Project,
        model::{ProjectEdit, TransactionKey},
    },
};

#[derive(Default)]
pub struct EditorUiState {
    pub settings: Option<SubmachineKey>,
    pub rename_buffer: String,
    pub create_open: bool,
    pub new_machine_name: String,
    pub error: Option<String>,
    pub nodes: super::node_inspector::NodeInspectorState,
    pub edges: super::edge_inspector::EdgeInspectorState,
}

pub fn diagram_editor_page(
    project: &Project,
    editor: &mut DiagramEditorSession,
    state: &mut EditorUiState,
    ctx: &egui::Context,
) -> Vec<Command> {
    let mut commands = Vec::new();
    egui::SidePanel::right("editor_controls")
        .resizable(false)
        .exact_width(216.)
        .show_separator_line(false)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            crate::ui::theme::sidebar(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    control_panel(project, ui, &mut commands);
                });
            });
        });
    egui::SidePanel::left("editor_tools")
        .resizable(false)
        .exact_width(216.)
        .show_separator_line(false)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            crate::ui::theme::sidebar(ui, |ui| {
                tools_panel(project, editor, state, ui, &mut commands);
            });
        });
    editor.expire_compile_error(ctx);
    if let Some(message) = editor.compile_error().map(str::to_owned) {
        egui::TopBottomPanel::top("compile_result").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(egui::Color32::DARK_RED, message);
                if ui.small_button("×").on_hover_text("Close").clicked() {
                    editor.clear_compile_error();
                }
            });
        });
    }
    let active = editor.active_submachine();
    let tool = editor.tool().clone();
    egui::CentralPanel::default()
        .frame(egui::Frame::default())
        .show(ctx, |ui| {
            let Some(original) = project.document().submachine(active) else {
                return;
            };
            let mut draft = original.clone();
            if original.is_system() {
                super::super::runner::page::render_read_only_canvas(
                    original,
                    project.document().alphabet(),
                    &mut editor.canvas_mut(active).viewport,
                    None,
                    None,
                    ctx,
                    ui,
                );
            } else {
                handle_canvas(
                    &mut draft,
                    editor.canvas_mut(active),
                    &tool,
                    project.document().alphabet().clone(),
                    ctx,
                    ui,
                    &mut state.nodes,
                    &mut state.edges,
                );
                if &draft != original {
                    commands.push(Command::Edit {
                        edit: ProjectEdit::ReplaceMachine {
                            key: active,
                            machine: Box::new(draft),
                        },
                        group: Some(TransactionKey::Canvas(active)),
                    });
                }
            }
        });
    dialogs(project, state, ctx, &mut commands);
    commands
}
