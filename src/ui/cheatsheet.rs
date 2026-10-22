use crate::commands::{CommandId, REGISTRY};

fn group(ui: &mut egui::Ui, title: &str, rows: &[(String, String)], widths: [f32; 3]) {
    for (index, (control, action)) in rows.iter().enumerate() {
        ui.add_sized(
            [widths[0], 22.],
            egui::Label::new(egui::RichText::new(if index == 0 { title } else { "" }).strong())
                .wrap(),
        );
        ui.add_sized([widths[1], 22.], egui::Label::new(control).wrap());
        ui.add_sized([widths[2], 22.], egui::Label::new(action).wrap());
        ui.end_row();
    }
}

fn command_rows(ids: &[CommandId]) -> Vec<(String, String)> {
    ids.iter()
        .filter_map(|id| {
            let info = REGISTRY.iter().find(|info| info.id == *id)?;
            Some((info.shortcut()?, info.label.into()))
        })
        .collect()
}

fn rows(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items
        .iter()
        .map(|(control, action)| ((*control).into(), (*action).into()))
        .collect()
}

pub fn show(ctx: &egui::Context, open: &mut bool, runner: bool) {
    let size = (ctx.content_rect().size() - egui::vec2(80., 80.)).max(egui::vec2(120., 120.));
    let response = egui::Modal::new(egui::Id::new("cheatsheet"))
        .frame(crate::ui::theme::card_frame().fill(crate::ui::theme::PANEL_BACKGROUND))
        .show(ctx, |ui| {
            ui.set_width((size.x - 48.).max(80.));
            ui.set_min_height((size.y - 48.).max(80.));
            ui.horizontal(|ui| {
                ui.heading(if runner { "Runner cheatsheet" } else { "Editor cheatsheet" });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close · Esc").clicked() { *open = false; }
                });
            });
            ui.add_space(12.);
            egui::ScrollArea::vertical()
                .max_height((size.y - 102.).max(30.))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let width = (ui.available_width() - 24.).max(60.);
                    let widths = [width * 0.18, width * 0.29, width * 0.53];
                    egui::Grid::new("cheatsheet_controls")
                        .with_row_color(|row, _| Some(if row == 0 {
                            egui::Color32::from_rgb(228, 235, 240)
                        } else if row % 2 == 0 {
                            egui::Color32::from_rgb(246, 248, 249)
                        } else {
                            egui::Color32::WHITE
                        }))
                        .num_columns(3)
                        .spacing([8., 5.])
                        .show(ui, |ui| {
                            for (label, width) in ["Group", "Key / mouse", "Action"].into_iter().zip(widths) {
                                ui.add_sized([width, 24.], egui::Label::new(egui::RichText::new(label).strong()));
                            }
                            ui.end_row();
                            let mut project = command_rows(&[CommandId::Save, CommandId::Import]);
                            if !runner {
                                project.extend(command_rows(&[CommandId::Undo, CommandId::Redo, CommandId::OpenRunner, CommandId::Validate, CommandId::AddMachine]));
                                project.push(("Ctrl/⌘+Shift+Z".into(), "Redo".into()));
                            }
                            group(ui, "Project", &project, widths);
                            if !runner {
                                group(ui, "Tools", &command_rows(&[CommandId::Move, CommandId::Link, CommandId::Note, CommandId::Start, CommandId::End, CommandId::Left, CommandId::Right, CommandId::Write]), widths);
                                group(ui, "Selection", &rows(&[
                                    ("Click an item", "Select a node, transition or note and open its inspector"),
                                    ("Shift/⌘+click", "Toggle an item in the selection"),
                                    ("Right mouse drag", "Select nodes and notes inside a rectangle"),
                                    ("WASD / Arrow keys", "Move selected nodes and notes by one grid cell; WASD selects tools when nothing is selected"),
                                    ("Ctrl/⌘+C / Ctrl/⌘+V", "Copy / paste nodes, notes and their internal transitions within this project; skip submachine calls"),
                                    ("Delete", "Delete selected nodes, transitions and notes; Undo restores them"),
                                    ("X / Delete tool", "Select the deletion tool, then click an item to remove it"),
                                ]), widths);
                                group(ui, "Placement", &rows(&[
                                    ("Tool + canvas click", "Place a node or note at the nearest grid point"),
                                    ("Link + source click + target click", "Connect two nodes; select the transition to edit its symbols"),
                                    ("Link + empty canvas click", "Cancel the pending connection"),
                                    ("Drag a node or note", "Move it, preview the snapped position, and release to snap to the grid"),
                                ]), widths);
                            }
                            let pan = if runner { "Scroll" } else { "Scroll / Shift+scroll" };
                            let pan_action = if runner { "Pan the canvas" } else { "Pan vertically / horizontally" };
                            group(ui, "Canvas", &rows(&[
                                ("Left mouse drag on empty canvas", "Pan the canvas"),
                                (pan, pan_action),
                                ("Ctrl+scroll / pinch", "Zoom around the pointer"),
                            ]), widths);
                            if runner {
                                group(ui, "Execution", &command_rows(&[CommandId::Run, CommandId::Pause, CommandId::Step, CommandId::Reset, CommandId::Stop]), widths);
                                group(ui, "Tape", &rows(&[
                                    ("Click a tape cell", "Edit its symbol or set the head at that index"),
                                    ("Tape arrows / Return to head", "Browse cells / follow the head"),
                                    ("Text field + Apply tape", "Replace the tape and restart execution"),
                                ]), widths);
                                group(ui, "Navigation", &rows(&[
                                    ("Click a submachine card", "View that machine and turn off execution following"),
                                    ("Follow step execution", "Follow the executing node and submachine"),
                                ]), widths);
                            }
                            group(ui, "Help and dialogs", &rows(&[
                                ("F1 / ?", "Open this cheatsheet"),
                                ("Escape / click outside", "Close a modal dialog"),
                            ]), widths);
                        });
                    ui.add_space(18.);
                    ui.separator();
                    ui.add_space(12.);
                    if runner {
                        ui.label(egui::RichText::new("Execution and tests").strong());
                        ui.label("The green node marks the current action; the green arrow previews its outgoing transition after that action. Run starts execution, Pause suspends it, Step performs one step, Reset restores the initial state, and Stop ends the run. Step interval sets the delay between steps. Editor returns to editing.");
                        ui.label("Use the gear beside a test to edit its initial and expected tapes. Check all six runs the saved tests with a step limit; Cancel checks stops the batch. Results compare symbols relative to the head.");
                        ui.label("Tape notation: λ means blank. Mark exactly one head cell with parentheses, for example 01(1)0; use (λ) for an empty tape.");
                    } else {
                        ui.label(egui::RichText::new("Notes, submachines and projects").strong());
                        ui.label("Select a note to edit its text in the Note window. Its center stays on the same grid point as you type.");
                        ui.label("Choose a submachine card to place a call. Use its arrow to open the machine and its gear to rename or delete it. System machines are read-only.");
                        ui.label("Validate checks the diagram. Run compiles it and opens the runner. Projects returns to the library; Save stores the library, and Export downloads the project as JSON.");
                    }
                    ui.add_space(10.);
                    ui.weak("Letter shortcuts, Space, Reset, Stop, Delete and arrow movement pause while you type. Modal dialogs block workspace shortcuts.");
                });
        });
    if response.should_close() {
        *open = false;
    }
}
