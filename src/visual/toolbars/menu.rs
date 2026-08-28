use egui::{CentralPanel, Grid, Id, ScrollArea};

use crate::{
    app::App, core::ValidatedAlphabet, project::ProjectAction,
};

#[derive(Clone, Default)]
struct NewProjectForm {
    name: String,
    alphabet: String,
    error: Option<String>
}

pub fn render_menu(app: &mut App, ctx: &egui::Context) {
    CentralPanel::default().show(ctx, |ui| {
        Grid::new("menu grid")
            .num_columns(2)
            .min_row_height(ui.available_height())
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_project_management_column(app, ui);
                });

                ui.vertical(|ui| {
                    render_turing_machine_info(ui);
                });
            });
    });
}


fn render_project_management_column(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(24.0);
    ui.heading("TMTinker — Projects");
    ui.add_space(16.0);

    let form_id = Id::new("new_project_form_state");

    let mut form = ui.data_mut(|mem| {
        mem.get_temp::<NewProjectForm>(form_id).unwrap_or_default()
    });

    let mut action_to_take: Option<ProjectAction> = None;

    ui.group(|ui| {
        ui.label("New project:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut form.name).on_hover_text("Project name");
            ui.text_edit_singleline(&mut form.alphabet).on_hover_text("Alphabet e.g. \" 01\"");
            if ui.button("Create project").clicked() {
                match ValidatedAlphabet::new(form.alphabet.clone().chars()) {
                    Ok(alphabet) => {
                        action_to_take = Some(ProjectAction::CreateProject((form.name.clone(), alphabet)));
                        form.error = None
                    }
                    Err(err) => {
                        form.error = Some(format!("Invalid alphabet: {}", err));
                    }
                };
            }
        });
    });

    if let Some(action) = action_to_take {
        app.execute_project_action(action);
    }

    if let Some(ref err_msg) = form.error {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(err_msg).color(egui::Color32::RED));
    }

    ui.data_mut(|mem| {
        mem.insert_temp(form_id, form);
    });

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    render_projects_list_scrollarea(app, ui);
}

fn render_projects_list_scrollarea(app: &mut App, ui: &mut egui::Ui) {
    ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            let mut action_to_take: Option<ProjectAction> = None;

            for (key, project) in app.projects.iter() {
                ui.horizontal(|ui| {
                    let (name, alphabet) = project.info();
                    ui.label(format!("{}", name));
                    ui.label(format!("{}", alphabet.take(16).collect::<String>()));
                    if ui.button(format!("{}", name)).clicked() {
                        action_to_take = Some(ProjectAction::LoadFromMemory(key));
                    }
                });
            }

            if let Some(action) = action_to_take {
                app.execute_project_action(action);
            }
        });
}

fn render_turing_machine_info(ui: &mut egui::Ui) {
    ui.label(
        "A Turing machine is a mathematical model of computation describing an abstract machine \
        that manipulates symbols on a strip of tape according to a table of rules. Despite the \
        model's simplicity, it is capable of implementing any computer algorithm.\n\n\
        The machine operates on an infinite memory tape divided into discrete cells, each of \
        which can hold a single symbol drawn from a finite set of symbols called the alphabet \
        of the machine. It has a \"head\" that, at any point in the machine's operation, is \
        positioned over one of these cells, and a \"state\" selected from a finite set of states.",
    );
}
