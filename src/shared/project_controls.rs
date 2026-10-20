use crate::{
    commands::{Command, CommandId},
    project::Project,
    ui::widgets::custom_buttons::tool_button,
};

pub fn control_panel(project: &Project, ui: &mut egui::Ui, commands: &mut Vec<Command>) {
    ui.heading("Project");
    ui.add_space(10.);
    ui.label(egui::RichText::new(project.name()).strong());
    ui.weak(if project.is_tu4() { "TU4" } else { "Diagram" });
    ui.add_space(14.);
    crate::ui::widgets::custom_buttons::centered_tool_grid(ui, "project_controls", |ui| {
        for (index, id) in [
            CommandId::OpenRunner,
            CommandId::Validate,
            CommandId::Undo,
            CommandId::Redo,
            CommandId::Save,
            CommandId::Export,
            CommandId::Menu,
            CommandId::Cheatsheet,
        ]
        .into_iter()
        .enumerate()
        {
            let enabled = match id {
                CommandId::Undo => project.can_undo(),
                CommandId::Redo => project.can_redo(),
                _ => true,
            };
            let info = id.info();
            let shortcut = info.shortcut();
            ui.add_enabled_ui(enabled, |ui| {
                let response = tool_button(
                    ui,
                    info.icon,
                    match id {
                        CommandId::OpenRunner => "Run",
                        CommandId::Export => "Export",
                        _ => info.label,
                    },
                    shortcut.as_deref(),
                    false,
                );
                let response = if let Some(shortcut) = &shortcut {
                    response.on_hover_text(shortcut)
                } else {
                    response
                };
                let label = match id {
                    CommandId::Undo => project.undo_label(),
                    CommandId::Redo => project.redo_label(),
                    _ => None,
                };
                let response = if let Some(label) = label {
                    response.on_hover_text(label)
                } else {
                    response
                };
                if response.clicked() {
                    commands.push(Command::Invoke(id));
                }
            });
            if index % 2 == 1 {
                ui.end_row();
            }
        }
    });
    if !project.is_tu4() && crate::commands::button(ui, CommandId::OpenTu4, true).clicked() {
        commands.push(Command::Invoke(CommandId::OpenTu4));
    }
    ui.add_space(16.);
    ui.separator();
    ui.add_space(8.);
    ui.weak("Changes save automatically.");
}
