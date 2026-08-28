use crate::{
    persistence::project_to_json::ProjectBundle, project::Project, visual::mappings::{
         BUTTON_SIZE, CONTROL_START, ICON_FONT_SIZE, OPTIONS_EXPORT, OPTIONS_HELP, OPTIONS_MENU, OPTIONS_RUN, OPTIONS_SAVE, OPTIONS_VALIDATE, SUBSCRIPT_FONT_SIZE,
    },
};
use egui::{RichText, Ui};

fn control_button(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    shortcut: Option<&str>,
) -> bool {
    let button_size = egui::vec2(BUTTON_SIZE, BUTTON_SIZE);
    let mut clicked = false;

    ui.allocate_ui_with_layout(
        egui::vec2(button_size.x, button_size.y + 25.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            let response = ui.add_sized(
                button_size,
                egui::Button::new(RichText::new(icon).size(ICON_FONT_SIZE).color(
                        ui.visuals().text_color()
                ))
                .frame(true),
            );

            if response.clicked() {
                clicked = true;
            }

            if let Some(shortcut) = shortcut {
                let rect = response.rect;

                let galley = ui.painter().layout_no_wrap(
                    shortcut.to_owned(),
                    egui::FontId::proportional(10.0),
                    ui.visuals().weak_text_color(),
                );

                let padding = 3.0;
                let text_pos = egui::pos2(
                    rect.right() - galley.size().x - padding,
                    rect.top() + padding,
                );

                ui.painter()
                    .galley(text_pos, galley, ui.visuals().weak_text_color());
            }

            ui.label(RichText::new(label).size(SUBSCRIPT_FONT_SIZE));
        },
    );

    clicked
}

pub enum ControlAction {
    None,
    Menu,
    Save,
    Run,
    Help,
}

pub fn control_window(project: &mut Project, ui: &mut Ui) -> ControlAction {
    let mut control_action = ControlAction::None;

    egui::Grid::new("control_buttons")
        .num_columns(2)
        .striped(false)
        .spacing(egui::vec2(15.0, 10.0))
        .show(ui, |ui| {

                if control_button(ui, OPTIONS_RUN, "Start", None) {
                    // TODO run page
                    // if !app.start_tm() {
                    //     app.file_status = Some("No Start node found!".into());
                }

                // if control_button(ui, CONTROL_RESET, "Reset", None) {
                //     // app.reset_tm();
                // }

                if control_button(ui, OPTIONS_VALIDATE, "Validate", None) {
                    project.validate_machines();
                }

                if control_button(ui, OPTIONS_SAVE, "Save", None) {
                    control_action = ControlAction::Save;
                }

                ui.end_row();

                if control_button(ui, OPTIONS_EXPORT, "Export", None) {
                    match ProjectBundle::get_bundle(&project).to_json() {
                        Ok(val) => println!("{}", val),
                        Err(_) => ()
                    }
                    project.validate_machines();
                }

                if control_button(ui, OPTIONS_HELP, "Help", None) {
                    control_action = ControlAction::Help;
                }

                if control_button(ui, OPTIONS_MENU, "Menu", None) {
                    control_action = ControlAction::Menu;
                }

                // if control_button(ui, app, CONTROL_STEP, "Step", None) {
                //     app.step_tm();
                // }
                // if app.is_running {
                //     if control_button(ui, app, CONTROL_PAUSE, "Pause", None) {
                //         app.is_running = false;
                //     }
                // } else if control_button(ui, app, CONTROL_RUN, "Run", None) {
                //     app.is_running = true;
                // }
                // ui.end_row();
                //
                // if control_button(ui, app, CONTROL_SKIP, "Skip", None) {
                //     app.skip_tm();
                // }
                // if control_button(ui, app, CONTROL_STOP, "Stop", None) {
                //     app.halt_tm();
                // }
                // ui.end_row();
        });

    control_action
}
