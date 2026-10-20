use crate::{
    commands::{Command, CommandId, button},
    core::{tape::Tape, test_case::TestCase},
    project::{Project, model::ProjectEdit},
    shared::{
        checks::BatchCheck,
        runtime::{RunSession, RunStatus},
    },
    ui::widgets::{
        custom_buttons::tool_button,
        tape::{TapeDraft, TapeView, notation_hint},
    },
};

pub struct SharedRunnerUiState {
    pub follow_execution: bool,
    pub fast: bool,
    pub interval_ms: u64,
    pub next_step: f64,
    pub tape_center: i32,
    pub follow_head: bool,
    selected_cell: Option<i32>,
    tape_draft: TapeDraft,
    tape_error: Option<String>,
    test_dialog: Option<usize>,
    initial_draft: TapeDraft,
    expected_draft: TapeDraft,
    test_error: Option<String>,
}
impl SharedRunnerUiState {
    pub fn new(tape: &Tape) -> Self {
        Self {
            follow_execution: true,
            fast: false,
            interval_ms: 350,
            next_step: 0.,
            tape_center: tape.head_position(),
            follow_head: true,
            selected_cell: None,
            tape_draft: TapeDraft::from_tape(tape),
            tape_error: None,
            test_dialog: None,
            initial_draft: TapeDraft::from_tape(tape),
            expected_draft: TapeDraft::from_tape(&Tape::new()),
            test_error: None,
        }
    }
    pub fn has_modal(&self) -> bool {
        self.test_dialog.is_some()
    }
    pub fn sync_tape(&mut self, tape: &Tape) {
        self.tape_draft = TapeDraft::from_tape(tape);
        self.tape_error = None;
        self.tape_center = tape.head_position();
        self.follow_head = true;
        self.selected_cell = None;
    }
}

pub fn tick<S: RunSession>(
    session: &mut S,
    state: &mut SharedRunnerUiState,
    now: f64,
    ctx: &egui::Context,
) {
    if session.status() != RunStatus::Running {
        return;
    }
    if state.fast {
        #[cfg(not(target_arch = "wasm32"))]
        let started = std::time::Instant::now();
        for _ in 0..4096 {
            if session.status() != RunStatus::Running {
                break;
            }
            #[cfg(not(target_arch = "wasm32"))]
            if started.elapsed() >= std::time::Duration::from_millis(4) {
                break;
            }
            session.step();
        }
        if session.status() == RunStatus::Running {
            ctx.request_repaint();
        }
    } else {
        if now >= state.next_step {
            session.step();
            state.next_step = now + state.interval_ms as f64 / 1000.;
        }
        if session.status() == RunStatus::Running {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(
                (state.next_step - now).max(0.001),
            ));
        }
    }
}

pub fn controls_panel<S: RunSession>(
    project: &Project,
    session: &S,
    state: &mut SharedRunnerUiState,
    checking: bool,
    ctx: &egui::Context,
    commands: &mut Vec<Command>,
    extra: impl FnOnce(&mut egui::Ui, &mut SharedRunnerUiState),
) {
    egui::SidePanel::right("runner_controls")
        .exact_width(216.)
        .resizable(false)
        .show_separator_line(false)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            crate::ui::theme::sidebar(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("run_controls_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(if project.is_tu4() {
                                "Run TU4"
                            } else {
                                "Run diagram"
                            });
                            if crate::commands::icon_button(ui, CommandId::Cheatsheet).clicked() {
                                commands.push(Command::Invoke(CommandId::Cheatsheet));
                            }
                        });
                        ui.label(egui::RichText::new(project.name()).strong());
                        ui.add_space(14.);
                        execution_buttons(ui, session.status(), checking, commands);
                        ui.add_space(12.);
                        ui.checkbox(&mut state.fast, "Run without delay");
                        ui.label("Step interval");
                        ui.spacing_mut().slider_width = (ui.available_width() - 80.).max(60.);
                        ui.scope(|ui| {
                            ui.visuals_mut().widgets.inactive.bg_fill =
                                crate::ui::theme::PANEL_STROKE;
                            ui.add(
                                egui::Slider::new(&mut state.interval_ms, 50..=2000)
                                    .suffix(" ms")
                                    .logarithmic(true),
                            );
                        });
                        ui.checkbox(&mut state.follow_execution, "Follow step execution");
                        ui.label(format!("Status: {:?}", session.status()));
                        ui.label(format!("Steps: {}", session.steps()));
                        if let Some(error) = session.fault() {
                            ui.colored_label(egui::Color32::DARK_RED, error);
                        }
                        ui.add_space(12.);
                        ui.separator();
                        ui.add_space(8.);
                        crate::ui::widgets::custom_buttons::centered_tool_grid(
                            ui,
                            "runner_menu",
                            |ui| {
                                for (i, id) in [CommandId::BackToEditor, CommandId::Save]
                                    .into_iter()
                                    .enumerate()
                                {
                                    let enabled = !checking;
                                    let info = id.info();
                                    ui.add_enabled_ui(enabled, |ui| {
                                        if tool_button(
                                            ui,
                                            info.icon,
                                            info.label,
                                            info.shortcut().as_deref(),
                                            false,
                                        )
                                        .on_hover_text(info.shortcut().unwrap_or_default())
                                        .clicked()
                                        {
                                            commands.push(Command::Invoke(id));
                                        }
                                    });
                                    if i % 2 == 1 {
                                        ui.end_row();
                                    }
                                }
                            },
                        );
                        extra(ui, state);
                    });
            });
        });
}
pub fn tests_panel<S: RunSession>(
    project: &Project,
    state: &mut SharedRunnerUiState,
    batch: Option<&BatchCheck<S>>,
    ctx: &egui::Context,
    commands: &mut Vec<Command>,
) {
    let checking = batch.is_some_and(|b| !b.is_done());
    egui::SidePanel::left("runner_tests")
        .exact_width(244.)
        .resizable(false)
        .show_separator_line(false)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            crate::ui::theme::sidebar(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("tests_scroll")
                    .show(ui, |ui| {
                        ui.heading("Tests");
                        ui.weak("Load a tape, then press Run.");
                        ui.add_space(12.);
                        for slot in 0..6 {
                            ui.push_id(slot, |ui| {
                                egui::Frame::new()
                                .fill(egui::Color32::from_rgb(246, 248, 249))
                                .stroke(egui::Stroke::new(1., crate::ui::theme::PANEL_STROKE))
                                .corner_radius(5.)
                                .inner_margin(8.)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let exists = project.data().tests.get(slot).expect("test slot is in range").is_some();
                                        if ui
                                            .add_enabled(
                                                !checking,
                                                egui::Button::new(format!(
                                                    "{}  {}",
                                                    slot + 1,
                                                    if exists {
                                                        "Load test"
                                                    } else {
                                                        "Set up test"
                                                    }
                                                ))
                                                .min_size(egui::vec2(136., 32.)),
                                            )
                                            .clicked()
                                        {
                                            if exists {
                                                commands.push(Command::LoadTest(slot));
                                            } else {
                                                open_test(state, project, slot);
                                            }
                                        }
                                        if ui
                                            .add_enabled(!checking, egui::Button::new("⚙"))
                                            .on_hover_text(CommandId::EditTest.info().label)
                                            .clicked()
                                        {
                                            open_test(state, project, slot);
                                        }
                                    });
                                    if let Some(batch) = batch {
                                        if let Some(result) = batch.results.get(slot).expect("test slot is in range") {
                                            let color = if matches!(
                                                result,
                                                crate::core::test_case::TestResult::Passed { .. }
                                            ) {
                                                egui::Color32::from_rgb(25, 130, 100)
                                            } else {
                                                egui::Color32::DARK_RED
                                            };
                                            ui.colored_label(color, result.label());
                                        } else if checking && batch.current_slot() == slot {
                                            ui.horizontal(|ui| {
                                                ui.spinner();
                                                ui.label("Checking…");
                                            });
                                        } else if project.data().tests.get(slot).expect("test slot is in range").is_none() {
                                            ui.weak("Empty");
                                        } else if batch.is_done() {
                                            ui.weak("Not checked");
                                        }
                                    } else {
                                        ui.weak(if project.data().tests.get(slot).expect("test slot is in range").is_some() {
                                            "Ready"
                                        } else {
                                            "Empty"
                                        });
                                    }
                                });
                            });
                            ui.add_space(8.);
                        }
                        ui.add_space(4.);
                        let id = if checking {
                            CommandId::CancelChecks
                        } else {
                            CommandId::CheckAll
                        };
                        if button(
                            ui,
                            id,
                            checking || project.data().tests.iter().any(Option::is_some),
                        )
                        .clicked()
                        {
                            commands.push(Command::Invoke(id));
                        }
                        ui.add_space(10.);
                        ui.label("Step limit per test");
                        let mut limit = project.data().step_limit;
                        if ui
                            .add_enabled(
                                !checking,
                                egui::DragValue::new(&mut limit)
                                    .range(1..=10_000_000)
                                    .speed(100),
                            )
                            .changed()
                        {
                            commands.push(Command::Edit {
                                edit: ProjectEdit::SetStepLimit(limit),
                                group: None,
                            });
                        }
                        ui.weak("Compare tape contents relative to the head.");
                    });
            });
        });
}
pub fn tape_panel<S: RunSession>(
    project: &Project,
    session: &S,
    state: &mut SharedRunnerUiState,
    checking: bool,
    ctx: &egui::Context,
    commands: &mut Vec<Command>,
) {
    egui::TopBottomPanel::top("runner_tape")
        .resizable(false)
        .show_separator_line(false)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            crate::ui::theme::panel_frame().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(6.);
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("Tape").strong());
                    for (id, delta) in [
                        (CommandId::TapeLeftFive, -5),
                        (CommandId::TapeLeft, -1),
                        (CommandId::TapeRight, 1),
                        (CommandId::TapeRightFive, 5),
                    ] {
                        if crate::commands::icon_button(ui, id).clicked() {
                            commands.push(Command::ScrollTape(delta));
                        }
                    }
                    if button(ui, CommandId::TapeHead, true).clicked() {
                        commands.push(Command::Invoke(CommandId::TapeHead));
                    }
                    ui.weak(format!("Head: {}", session.tape().head_position()));
                });
                if state.follow_head {
                    state.tape_center = session.tape().head_position();
                }
                let radius = (((ui.available_width() / 46.).floor() as i32).clamp(1, 101) - 1) / 2;
                let first = i64::from(state.tape_center) - i64::from(radius);
                let response = ui.add(TapeView::new(session.tape()).centered(
                    state.tape_center,
                    radius,
                    state.selected_cell,
                ));
                if response.clicked() && !checking {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let index = first + ((pos.x - response.rect.left()) / 46.).floor() as i64;
                        if let Ok(index) = i32::try_from(index) {
                            state.selected_cell = Some(index);
                        }
                    }
                }
                ui.add_enabled_ui(!checking, |ui| {
                    if let Some(index) = state.selected_cell {
                        ui.horizontal(|ui| {
                            ui.label(format!("Cell {index}"));
                            let mut symbol = session.tape().cell(index);
                            let previous = symbol;
                            egui::ComboBox::from_id_salt("cell_symbol")
                                .selected_text(if symbol == ' ' {
                                    "λ".into()
                                } else {
                                    symbol.to_string()
                                })
                                .show_ui(ui, |ui| {
                                    for &c in project.alphabet().iter() {
                                        ui.selectable_value(
                                            &mut symbol,
                                            c,
                                            if c == ' ' { "λ".into() } else { c.to_string() },
                                        );
                                    }
                                });
                            if symbol != previous {
                                let mut tape = session.tape().clone();
                                tape.set_cell(index, symbol);
                                commands.push(Command::SetTape(tape));
                            }
                            if ui.button("Set head here").clicked() {
                                let mut tape = session.tape().clone();
                                tape.set_head(index);
                                commands.push(Command::SetTape(tape));
                            }
                            if ui.small_button("Close").clicked() {
                                state.selected_cell = None;
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        let height = ui.spacing().interact_size.y.max(32.);
                        ui.add_sized([100., height], egui::Label::new("Edit tape as text"));
                        let button_width = 90.;
                        ui.add_sized(
                            [
                                (ui.available_width() - button_width - ui.spacing().item_spacing.x)
                                    .max(40.),
                                height,
                            ],
                            egui::TextEdit::singleline(&mut state.tape_draft.text)
                                .font(egui::TextStyle::Body),
                        );
                        if ui
                            .add_sized([button_width, height], egui::Button::new("Apply tape"))
                            .clicked()
                        {
                            match state.tape_draft.parse(project.alphabet()) {
                                Ok(tape) => {
                                    commands.push(Command::SetTape(tape));
                                    state.tape_error = None;
                                }
                                Err(e) => state.tape_error = Some(e),
                            }
                        }
                    });
                    if let Some(error) = &state.tape_error {
                        ui.colored_label(egui::Color32::DARK_RED, error);
                    }
                });
                ui.add_space(4.);
            });
        });
}
fn open_test(state: &mut SharedRunnerUiState, project: &Project, slot: usize) {
    let empty = TestCase {
        initial_tape: project.data().prepared_tape.clone(),
        expected_tape: Tape::new(),
    };
    let test = project
        .data()
        .tests
        .get(slot)
        .expect("test slot is in range")
        .as_ref()
        .unwrap_or(&empty);
    state.initial_draft = TapeDraft::from_tape(&test.initial_tape);
    state.expected_draft = TapeDraft::from_tape(&test.expected_tape);
    state.test_dialog = Some(slot);
    state.test_error = None;
}
pub fn test_dialog<S: RunSession>(
    project: &Project,
    session: &S,
    state: &mut SharedRunnerUiState,
    ctx: &egui::Context,
    commands: &mut Vec<Command>,
) {
    let Some(slot) = state.test_dialog else {
        return;
    };
    let response = egui::Modal::new(egui::Id::new("test_editor")).show(ctx, |ui| {
        ui.set_width(420.);
        ui.heading(format!("Test {}", slot + 1));
        ui.add_space(12.);
        ui.label(egui::RichText::new("Initial tape").strong());
        state.initial_draft.show(ui);
        ui.add_space(12.);
        ui.label(egui::RichText::new("Expected tape").strong());
        state.expected_draft.show(ui);
        if ui.button("Use current tape as expected").clicked() {
            state.expected_draft = TapeDraft::from_tape(session.tape());
        }
        notation_hint(ui);
        ui.weak("Editing a tape restarts execution with the edited tape.");
        if let Some(error) = &state.test_error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
        ui.add_space(12.);
        ui.horizontal(|ui| {
            if ui.button("Save test").clicked() {
                let result = (|| {
                    Ok::<_, String>(TestCase {
                        initial_tape: state.initial_draft.parse(project.alphabet())?,
                        expected_tape: state.expected_draft.parse(project.alphabet())?,
                    })
                })();
                match result {
                    Ok(test) => {
                        commands.push(Command::Edit {
                            edit: ProjectEdit::SetTest {
                                slot,
                                test: Some(test),
                            },
                            group: None,
                        });
                        state.test_dialog = None;
                    }
                    Err(e) => state.test_error = Some(e),
                }
            }
            if ui
                .add_enabled(
                    project
                        .data()
                        .tests
                        .get(slot)
                        .expect("test slot is in range")
                        .is_some(),
                    egui::Button::new("Clear slot"),
                )
                .clicked()
            {
                commands.push(Command::Edit {
                    edit: ProjectEdit::SetTest { slot, test: None },
                    group: None,
                });
                state.test_dialog = None;
            }
            if ui.button("Cancel").clicked() {
                state.test_dialog = None;
            }
        });
    });
    if response.should_close() {
        state.test_dialog = None;
    }
}

fn execution_buttons(
    ui: &mut egui::Ui,
    status: RunStatus,
    checking: bool,
    commands: &mut Vec<Command>,
) {
    crate::ui::widgets::custom_buttons::centered_tool_grid(ui, "runner_buttons", |ui| {
        let primary = if status == RunStatus::Running {
            CommandId::Pause
        } else {
            CommandId::Run
        };
        for (i, id) in [primary, CommandId::Step, CommandId::Reset, CommandId::Stop]
            .into_iter()
            .enumerate()
        {
            let info = id.info();
            let enabled = !checking
                && (matches!(id, CommandId::Reset | CommandId::Stop | CommandId::Pause)
                    || status == RunStatus::Paused);
            ui.add_enabled_ui(enabled, |ui| {
                if tool_button(ui, info.icon, info.label, info.shortcut().as_deref(), false)
                    .clicked()
                {
                    commands.push(Command::Invoke(id));
                }
            });
            if i % 2 == 1 {
                ui.end_row();
            }
        }
    });
}
