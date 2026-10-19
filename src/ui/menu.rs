use super::menu_background::MenuBackground;
use crate::{
    app::ProjectKey,
    commands::{Command, CommandId, button},
    core::alphabet::ValidatedAlphabet,
    persistence::creation_date,
    project::{Project, model::ProjectName},
};
use egui::{Color32, RichText};
use slotmap::SlotMap;

pub struct MenuUiState {
    pub new_project_open: bool,
    pub delete_project: Option<ProjectKey>,
    name: String,
    alphabet: String,
    error: Option<String>,
    background: MenuBackground,
}
impl Default for MenuUiState {
    fn default() -> Self {
        Self {
            new_project_open: false,
            delete_project: None,
            name: String::new(),
            alphabet: "01".into(),
            error: None,
            background: MenuBackground::default(),
        }
    }
}

pub fn render_menu(
    projects: &SlotMap<ProjectKey, Project>,
    state: &mut MenuUiState,
    ctx: &egui::Context,
) -> Vec<Command> {
    let mut commands = Vec::new();
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let screen = ui.max_rect();
            state.background.show(ctx, screen);

            let margin = (screen.width().min(screen.height()) * 0.065).clamp(12., 48.);
            let card = screen.shrink(margin);
            ui.scope_builder(egui::UiBuilder::new().max_rect(card), |ui| {
                crate::ui::theme::card_frame().show(ui, |ui| {
                    ui.set_min_size((card.size() - egui::vec2(50., 50.)).max(egui::Vec2::ZERO));
                    ui.label(RichText::new("TMTinker").size(32.).strong());
                    ui.label(
                        RichText::new("Create and run Turing machine diagrams.")
                            .color(Color32::from_gray(105)),
                    );
                    ui.add_space(22.);
                    let height = (ui.available_height() - 45.).max(80.);
                    if ui.available_width() >= 700. {
                        let width = ui.available_width();
                        ui.horizontal_top(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(width * 0.59, height),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    project_list(projects, state, ui, height, &mut commands);
                                },
                            );
                            ui.add_space(24.);
                            ui.vertical(|ui| {
                                ui.set_max_width((width * 0.35).max(100.));
                                machine_info(ui);
                            });
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(height)
                            .show(ui, |ui| {
                                project_list(
                                    projects,
                                    state,
                                    ui,
                                    (height * 0.64).max(120.),
                                    &mut commands,
                                );
                                ui.add_space(16.);
                                machine_info(ui);
                            });
                    }
                });
            });
        });
    egui::Area::new(egui::Id::new("external_links"))
        .anchor(egui::Align2::RIGHT_BOTTOM, [-12., -10.])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (label, url, tooltip) in [
                    (
                        "GH",
                        "https://github.com/nh4tco4/TMTinker",
                        "TMTinker on GitHub",
                    ),
                    ("F8", "https://faq8.ru/", "faq8.ru"),
                ] {
                    if ui
                        .add_sized([36., 36.], egui::Button::new(RichText::new(label).strong()))
                        .on_hover_text(tooltip)
                        .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                    }
                }
            });
        });
    new_project_dialog(state, ctx, &mut commands);
    if let Some(key) = state.delete_project {
        if let Some(project) = projects.get(key) {
            let response = egui::Modal::new(egui::Id::new("delete_project")).show(ctx, |ui| {
                ui.set_width(360.);
                ui.heading("Delete project?");
                ui.label(format!(
                    "Delete “{}” and its saved diagrams and tests? This cannot be undone.",
                    project.name()
                ));
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        state.delete_project = None;
                    }
                    if ui.button("Delete project").clicked() {
                        commands.push(Command::DeleteProject(key));
                        state.delete_project = None;
                    }
                });
            });
            if response.should_close() {
                state.delete_project = None;
            }
        } else {
            state.delete_project = None;
        }
    }
    commands
}

fn project_list(
    projects: &SlotMap<ProjectKey, Project>,
    state: &mut MenuUiState,
    ui: &mut egui::Ui,
    height: f32,
    commands: &mut Vec<Command>,
) {
    ui.horizontal(|ui| {
        ui.heading("Your projects");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if button(ui, CommandId::NewProject, true).clicked() {
                commands.push(Command::Invoke(CommandId::NewProject));
            }
        });
    });
    ui.add_space(8.);
    if button(ui, CommandId::Import, true).clicked() {
        commands.push(Command::Invoke(CommandId::Import));
    }
    ui.add_space(12.);
    egui::ScrollArea::vertical()
        .id_salt("projects")
        .max_height((height - 90.).max(40.))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if projects.is_empty() {
                ui.add_space(24.);
                ui.label(RichText::new("Your first machine starts here").size(18.));
                ui.weak("Create a diagram or import a project file.");
            }
            for (key, project) in projects {
                egui::Frame::new()
                    .fill(Color32::from_rgb(246, 248, 249))
                    .stroke(egui::Stroke::new(1., Color32::from_rgb(220, 225, 228)))
                    .corner_radius(8.)
                    .inner_margin(12.)
                    .show(ui, |ui| {
                        ui.set_width((ui.available_width() - 2.).max(0.));
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new(project.name()).size(17.).strong());
                                ui.weak(format!(
                                    "Diagram · {} symbols",
                                    project.document().alphabet().len()
                                ));
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if button(ui, CommandId::OpenProject, true).clicked() {
                                        commands.push(Command::Open(key));
                                    }
                                    if button(ui, CommandId::DeleteProject, true).clicked() {
                                        state.delete_project = Some(key);
                                    }
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new("?")
                                                .size(19.)
                                                .color(Color32::from_gray(140)),
                                        )
                                        .selectable(false)
                                        .sense(egui::Sense::hover()),
                                    )
                                    .on_hover_ui(|ui| {
                                        ui.label(format!(
                                            "Alphabet: {}",
                                            project
                                                .document()
                                                .alphabet()
                                                .iter()
                                                .map(|c| if *c == ' ' { 'λ' } else { *c })
                                                .collect::<String>()
                                        ));
                                        ui.label(format!(
                                            "Created: {}",
                                            creation_date(project.data().created_at)
                                        ));
                                    });
                                },
                            );
                        });
                    });
                ui.add_space(8.);
            }
        });
}
fn machine_info(ui: &mut egui::Ui) {
    ui.heading("About Turing machines");
    ui.add_space(14.);
    ui.label("A Turing machine reads and writes symbols on an unbounded tape. Its current state and the symbol under its head determine the next action.");
    ui.add_space(12.);
    ui.label("Connect nodes to describe an algorithm, reuse submachines, then watch it run one step at a time.");
    ui.add_space(22.);
    ui.separator();
    ui.add_space(12.);
    ui.add_enabled(false, egui::Button::new("Documentation · Soon"));
}

fn new_project_dialog(state: &mut MenuUiState, ctx: &egui::Context, commands: &mut Vec<Command>) {
    if state.new_project_open {
        let response = egui::Modal::new(egui::Id::new("new_project")).show(ctx, |ui| {
            ui.set_width(360.);
            ui.heading("New project");
            ui.add_space(12.);
            ui.label("Project type");
            ui.horizontal(|ui| {
                let _ = ui.selectable_label(true, "Diagram");
                ui.add_enabled(false, egui::Button::new("Quadruples · Soon"));
            });
            ui.add_space(12.);
            ui.label("Name");
            ui.text_edit_singleline(&mut state.name);
            ui.label("Alphabet");
            ui.text_edit_singleline(&mut state.alphabet);
            ui.weak("The blank symbol λ is included automatically.");
            if let Some(error) = &state.error {
                ui.colored_label(Color32::from_rgb(180, 45, 50), error);
            }
            ui.add_space(16.);
            ui.horizontal(|ui| {
                if ui.button("Create project").clicked() {
                    let result = (|| {
                        let name =
                            ProjectName::new(state.name.clone()).map_err(|e| e.to_string())?;
                        let mut chars: Vec<_> = state
                            .alphabet
                            .chars()
                            .map(|c| if c == 'λ' { ' ' } else { c })
                            .collect();
                        if !chars.contains(&' ') {
                            chars.insert(0, ' ');
                        }
                        let alphabet = ValidatedAlphabet::new(chars).map_err(|e| e.to_string())?;
                        Ok::<_, String>(Command::Create { name, alphabet })
                    })();
                    match result {
                        Ok(command) => {
                            commands.push(command);
                            state.new_project_open = false;
                            state.name.clear();
                            state.error = None;
                        }
                        Err(error) => state.error = Some(error),
                    }
                }
                if ui.button("Cancel").clicked() {
                    state.new_project_open = false;
                }
            });
        });
        if response.should_close() {
            state.new_project_open = false;
        }
    }
}
