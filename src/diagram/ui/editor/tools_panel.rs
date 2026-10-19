use super::page::EditorUiState;
use crate::{
    commands::{Command, CommandId},
    diagram::{
        editor::{
            session::DiagramEditorSession,
            tool::{BasicActionTools, EditTools, EditorTool},
        },
        model::submachine::SubmachineKey,
    },
    project::{Project, model::ProjectEdit},
    ui::{
        theme::*,
        widgets::custom_buttons::{icon_button, tool_button},
    },
};

pub fn tool_for_command(id: CommandId) -> Option<EditorTool> {
    use CommandId::*;
    Some(match id {
        Move => EditorTool::Edit(EditTools::Move),
        Link => EditorTool::Edit(EditTools::Link),
        Note => EditorTool::Edit(EditTools::Comment),
        Delete => EditorTool::Edit(EditTools::Delete),
        Start => EditorTool::Place(BasicActionTools::Start),
        End => EditorTool::Place(BasicActionTools::Stop),
        Left => EditorTool::Place(BasicActionTools::Left),
        Right => EditorTool::Place(BasicActionTools::Right),
        Write => EditorTool::Place(BasicActionTools::Write),
        _ => return None,
    })
}

pub fn tools_panel(
    project: &Project,
    editor: &DiagramEditorSession,
    state: &mut EditorUiState,
    ui: &mut egui::Ui,
    commands: &mut Vec<Command>,
) {
    let locked = project
        .document()
        .submachine(editor.active_submachine())
        .is_some_and(|m| m.is_system());
    ui.heading("Tools");
    ui.add_space(12.);
    egui::ScrollArea::vertical()
        .id_salt("editor_tools_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            tool_group(
                ui,
                &[
                    CommandId::Move,
                    CommandId::Link,
                    CommandId::Note,
                    CommandId::Delete,
                ],
                editor,
                locked,
                commands,
            );
            ui.add_space(10.);
            ui.separator();
            ui.add_space(10.);
            tool_group(
                ui,
                &[
                    CommandId::Start,
                    CommandId::End,
                    CommandId::Left,
                    CommandId::Right,
                    CommandId::Write,
                ],
                editor,
                locked,
                commands,
            );
            ui.add_space(10.);
            ui.separator();
            ui.add_space(12.);
            ui.label(egui::RichText::new("Submachines").strong());
            ui.add_space(8.);
            for (key, machine) in project.document().submachines() {
                let is_active = key == editor.active_submachine();
                let selected = *editor.tool() == EditorTool::Submachine(key);
                let response = submachine_card(
                    ui,
                    key,
                    machine.machine().name(),
                    machine.is_system(),
                    is_active,
                    selected,
                    false,
                );
                if response.0 && !is_active && !locked {
                    commands.push(Command::SelectTool(EditorTool::Submachine(key)));
                }
                if response.1 {
                    commands.push(Command::OpenMachine(key));
                }
                if response.2 {
                    state.settings = Some(key);
                    state.rename_buffer = machine.machine().name().into();
                    state.error = None;
                }
                ui.add_space(8.);
            }
            if ui
                .add_sized(
                    [ui.available_width(), 40.],
                    egui::Button::new(format!(
                        "+   {}",
                        CommandId::AddMachine.info().shortcut().unwrap()
                    )),
                )
                .on_hover_text(CommandId::AddMachine.info().label)
                .clicked()
            {
                state.create_open = true;
                state.new_machine_name.clear();
                state.error = None;
            }
        });
}

fn tool_group(
    ui: &mut egui::Ui,
    ids: &[CommandId],
    editor: &DiagramEditorSession,
    locked: bool,
    commands: &mut Vec<Command>,
) {
    crate::ui::widgets::custom_buttons::centered_tool_grid(ui, ("tool_group", ids[0]), |ui| {
        for (index, &id) in ids.iter().enumerate() {
            let tool = tool_for_command(id).expect("tool command");
            let info = id.info();
            let shortcut = info.shortcut();
            ui.add_enabled_ui(!locked || id == CommandId::Move, |ui| {
                if tool_button(
                    ui,
                    info.icon,
                    info.label,
                    shortcut.as_deref(),
                    *editor.tool() == tool,
                )
                .clicked()
                {
                    commands.push(Command::SelectTool(tool));
                }
            });
            if index % 2 == 1 {
                ui.end_row();
            }
        }
    });
}

pub(crate) fn submachine_card(
    ui: &mut egui::Ui,
    key: SubmachineKey,
    name: &str,
    system: bool,
    active: bool,
    selected: bool,
    navigation_only: bool,
) -> (bool, bool, bool) {
    let mut result = (false, false, false);
    let width = ui.available_width();
    ui.push_id(key, |ui| {
        egui::Frame::new()
            .fill(if selected {
                egui::Color32::from_rgb(228, 248, 244)
            } else {
                egui::Color32::from_rgb(246, 248, 249)
            })
            .stroke(egui::Stroke::new(
                if selected { 2. } else { 1. },
                if selected {
                    SELECTED_NODE_STROKE
                } else {
                    PANEL_STROKE
                },
            ))
            .corner_radius(BUTTON_CORNER_RADIUS)
            .inner_margin(4.)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4., 4.);
                ui.horizontal(|ui| {
                    let label = format!("{}{}", if system { "🔒 " } else { "" }, name);
                    result.0 = ui
                        .add_enabled(
                            navigation_only || !active,
                            egui::Button::new(label).min_size(egui::vec2(
                                (width - if navigation_only { 10. } else { 52. }).max(45.),
                                64.,
                            )),
                        )
                        .on_hover_text(if navigation_only {
                            "View submachine"
                        } else if active {
                            "Current submachine"
                        } else {
                            CommandId::PlaceMachine.info().label
                        })
                        .clicked();
                    if !navigation_only {
                        ui.vertical(|ui| {
                            result.1 = icon_button(ui, "↗", egui::vec2(28., 30.))
                                .on_hover_text(CommandId::OpenMachine.info().label)
                                .clicked();
                            result.2 = icon_button(ui, "⚙", egui::vec2(28., 30.))
                                .on_hover_text(CommandId::MachineSettings.info().label)
                                .clicked();
                        });
                    }
                });
            });
    });
    result
}

pub fn dialogs(
    project: &Project,
    state: &mut EditorUiState,
    ctx: &egui::Context,
    commands: &mut Vec<Command>,
) {
    if state.create_open {
        let response = egui::Modal::new(egui::Id::new("create_machine")).show(ctx, |ui| {
            ui.set_width(300.);
            ui.heading("New submachine");
            ui.add_space(12.);
            ui.label("Name");
            ui.text_edit_singleline(&mut state.new_machine_name);
            if let Some(error) = &state.error {
                ui.colored_label(egui::Color32::DARK_RED, error);
            }
            ui.add_space(12.);
            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    match crate::diagram::model::submachine::SubmachineName::parse(
                        state.new_machine_name.clone(),
                    ) {
                        Ok(name) => {
                            commands.push(Command::Edit {
                                edit: ProjectEdit::AddMachine(name),
                                group: None,
                            });
                            state.create_open = false;
                        }
                        Err(error) => state.error = Some(error.to_string()),
                    }
                }
                if ui.button("Cancel").clicked() {
                    state.create_open = false;
                }
            });
        });
        if response.should_close() {
            state.create_open = false;
        }
    }
    if let Some(key) = state.settings {
        let Some(machine) = project.document().submachine(key) else {
            state.settings = None;
            return;
        };
        let response = egui::Modal::new(egui::Id::new("machine_settings")).show(ctx, |ui| {
            ui.set_width(320.);
            ui.heading("Submachine settings");
            ui.add_space(12.);
            ui.label("Name");
            ui.add_enabled(
                !machine.is_system(),
                egui::TextEdit::singleline(&mut state.rename_buffer),
            );
            let mut locked = machine.is_system();
            ui.add_enabled(
                false,
                egui::Checkbox::new(&mut locked, "System machine · read-only"),
            );
            if locked {
                ui.weak("System machines cannot be edited, renamed or unlocked.");
            }
            if let Some(error) = &state.error {
                ui.colored_label(egui::Color32::DARK_RED, error);
            }
            ui.add_space(12.);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!locked, egui::Button::new("Save name"))
                    .clicked()
                {
                    match crate::diagram::model::submachine::SubmachineName::parse(
                        state.rename_buffer.clone(),
                    ) {
                        Ok(name) => {
                            commands.push(Command::Edit {
                                edit: ProjectEdit::RenameMachine { key, name },
                                group: None,
                            });
                            state.settings = None;
                        }
                        Err(error) => state.error = Some(error.to_string()),
                    }
                }
                ui.add_enabled(false, egui::Button::new("Export · Soon"));
                if ui.button("Close").clicked() {
                    state.settings = None;
                }
            });
            ui.add_space(12.);
            ui.separator();
            let removal_error = project.document().can_remove_submachine(key).err();
            let delete = ui.add_enabled(
                removal_error.is_none(),
                egui::Button::new(
                    egui::RichText::new("Delete submachine").color(egui::Color32::DARK_RED),
                ),
            );
            if let Some(reason) = removal_error {
                delete.on_disabled_hover_text(&reason);
                ui.weak(reason);
            } else if delete
                .on_hover_text("Delete this submachine. Undo restores it.")
                .clicked()
            {
                commands.push(Command::Edit {
                    edit: ProjectEdit::RemoveMachine(key),
                    group: None,
                });
                state.settings = None;
            }
        });
        if response.should_close() {
            state.settings = None;
        }
    }
}
