use crate::{
    commands::{Command, CommandId, button},
    core::{alphabet::ValidatedAlphabet, tape::Tape, test_case::TestCase},
    diagram::{
        editor::session::{SelectionState, ViewportState, Zoom},
        model::{document::SubmachineDocument, layout::LayoutPos, submachine::SubmachineKey},
        runtime::{DiagramRunSession, RunStatus, checks::BatchCheck},
        ui::canvas::render::{
            render_edge::render_edges, render_grid::render_grid, render_node::render_nodes,
            render_note::render_notes,
        },
    },
    project::{Project, model::ProjectEdit},
    ui::widgets::{
        custom_buttons::tool_button,
        tape::{TapeDraft, TapeView, notation_hint},
    },
};
use slotmap::SecondaryMap;

pub struct RunnerUiState {
    pub active_submachine: SubmachineKey,
    viewports: SecondaryMap<SubmachineKey, ViewportState>,
    canvas_top: f32,
    canvas_rect: Option<egui::Rect>,
    followed_step: Option<(
        u64,
        SubmachineKey,
        crate::diagram::model::submachine::NodeKey,
    )>,
    follow_execution: bool,
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
impl RunnerUiState {
    pub fn has_modal(&self) -> bool {
        self.test_dialog.is_some()
    }
    pub fn new(active: SubmachineKey, tape: &Tape) -> Self {
        Self {
            active_submachine: active,
            viewports: SecondaryMap::new(),
            canvas_top: 0.,
            canvas_rect: None,
            followed_step: None,
            follow_execution: true,
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
    fn viewport_mut(&mut self, canvas_top: f32) -> &mut ViewportState {
        let offset = canvas_top - self.canvas_top;
        for viewport in self.viewports.values_mut() {
            viewport.camera_pan.y += offset;
        }
        self.canvas_top = canvas_top;
        self.viewports
            .entry(self.active_submachine)
            .expect("active submachine")
            .or_insert_with(|| {
                let mut viewport = ViewportState::new();
                viewport.camera_pan.y = canvas_top;
                viewport
            })
    }
    pub fn sync_tape(&mut self, tape: &Tape) {
        self.tape_draft = TapeDraft::from_tape(tape);
        self.tape_error = None;
        self.tape_center = tape.head_position();
        self.follow_head = true;
        self.selected_cell = None;
    }
}

pub fn diagram_runner_page(
    project: &Project,
    session: &DiagramRunSession,
    state: &mut RunnerUiState,
    batch: Option<&BatchCheck>,
    ctx: &egui::Context,
) -> Vec<Command> {
    let mut commands = Vec::new();
    let checking = batch.is_some_and(|b| !b.is_done());
    controls_panel(project, session, state, checking, ctx, &mut commands);
    tests_panel(project, state, batch, ctx, &mut commands);
    tape_panel(project, session, state, checking, ctx, &mut commands);
    runner_canvas(session, state, ctx);
    test_dialog(project, session, state, ctx, &mut commands);
    commands
}
fn controls_panel(
    project: &Project,
    session: &DiagramRunSession,
    state: &mut RunnerUiState,
    checking: bool,
    ctx: &egui::Context,
    commands: &mut Vec<Command>,
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
                            ui.heading("Run diagram");
                            if crate::commands::icon_button(ui, CommandId::Cheatsheet).clicked() {
                                commands.push(Command::Invoke(CommandId::Cheatsheet));
                            }
                        });
                        ui.label(egui::RichText::new(project.name()).strong());
                        ui.add_space(14.);
                        crate::ui::widgets::custom_buttons::centered_tool_grid(
                            ui,
                            "runner_buttons",
                            |ui| {
                                let primary = if session.status() == RunStatus::Running {
                                    CommandId::Pause
                                } else {
                                    CommandId::Run
                                };
                                for (i, id) in
                                    [primary, CommandId::Step, CommandId::Reset, CommandId::Stop]
                                        .into_iter()
                                        .enumerate()
                                {
                                    let info = id.info();
                                    let enabled = !checking
                                        && (matches!(
                                            id,
                                            CommandId::Reset | CommandId::Stop | CommandId::Pause
                                        ) || session.status() == RunStatus::Paused);
                                    ui.add_enabled_ui(enabled, |ui| {
                                        if tool_button(
                                            ui,
                                            info.icon,
                                            info.label,
                                            info.shortcut().as_deref(),
                                            false,
                                        )
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
                        ui.add_space(12.);
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
                        if ui
                            .checkbox(&mut state.follow_execution, "Follow step execution")
                            .changed()
                        {
                            state.followed_step = None;
                        }
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
                        ui.add_space(12.);
                        ui.separator();
                        ui.add_space(8.);
                        ui.label(egui::RichText::new("Submachines").strong());
                        for (key, machine) in session.compiled().source().submachines() {
                            let clicked = crate::diagram::ui::editor::tools_panel::submachine_card(
                                ui,
                                key,
                                machine.machine().name(),
                                machine.is_system(),
                                state.active_submachine == key,
                                state.active_submachine == key,
                                true,
                            )
                            .0;
                            ui.add_space(8.);
                            if clicked {
                                state.active_submachine = key;
                                state.follow_execution = false;
                            }
                        }
                    });
            });
        });
}
fn tests_panel(
    project: &Project,
    state: &mut RunnerUiState,
    batch: Option<&BatchCheck>,
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
                                        let exists = project.data().tests[slot].is_some();
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
                                        if let Some(result) = &batch.results[slot] {
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
                                        } else if project.data().tests[slot].is_none() {
                                            ui.weak("Empty");
                                        } else if batch.is_done() {
                                            ui.weak("Not checked");
                                        }
                                    } else {
                                        ui.weak(if project.data().tests[slot].is_some() {
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
fn tape_panel(
    project: &Project,
    session: &DiagramRunSession,
    state: &mut RunnerUiState,
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
                                    for &c in project.document().alphabet().iter() {
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
                            match state.tape_draft.parse(project.document().alphabet()) {
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
fn runner_canvas(session: &DiagramRunSession, state: &mut RunnerUiState, ctx: &egui::Context) {
    let location = session.source_location();
    if state.follow_execution {
        state.active_submachine = location.submachine;
    }
    egui::CentralPanel::default()
        .frame(egui::Frame::new())
        .show(ctx, |ui| {
            let active = state.active_submachine;
            let executing = (active == location.submachine).then_some(location.node);
            if let Some(document) = session.compiled().source().submachine(active) {
                let follow_key = (session.steps(), location.submachine, location.node);
                let canvas_rect = ui.max_rect();
                let recenter = state.follow_execution
                    && (state.followed_step != Some(follow_key)
                        || state.canvas_rect != Some(canvas_rect));
                state.canvas_rect = Some(canvas_rect);
                let center: LayoutPos = ui.max_rect().center().into();
                let viewport = state.viewport_mut(ui.max_rect().top());
                if recenter
                    && let Some(position) = document.layout().get_node_position(location.node)
                {
                    viewport.camera_pan = center - position * viewport.camera_zoom.get();
                }
                render_read_only_canvas(
                    document,
                    session.compiled().source().alphabet(),
                    viewport,
                    executing,
                    (active == location.submachine)
                        .then(|| session.next_transition())
                        .flatten(),
                    ctx,
                    ui,
                );
                if recenter {
                    state.followed_step = Some(follow_key);
                }
            }
        });
}

fn open_test(state: &mut RunnerUiState, project: &Project, slot: usize) {
    let empty = TestCase {
        initial_tape: project.data().prepared_tape.clone(),
        expected_tape: Tape::new(),
    };
    let test = project.data().tests[slot].as_ref().unwrap_or(&empty);
    state.initial_draft = TapeDraft::from_tape(&test.initial_tape);
    state.expected_draft = TapeDraft::from_tape(&test.expected_tape);
    state.test_dialog = Some(slot);
    state.test_error = None;
}
fn test_dialog(
    project: &Project,
    session: &DiagramRunSession,
    state: &mut RunnerUiState,
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
                        initial_tape: state.initial_draft.parse(project.document().alphabet())?,
                        expected_tape: state.expected_draft.parse(project.document().alphabet())?,
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
                    project.data().tests[slot].is_some(),
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

pub(crate) fn render_read_only_canvas(
    document: &SubmachineDocument,
    alphabet: &ValidatedAlphabet,
    viewport: &mut ViewportState,
    executing_node: Option<crate::diagram::model::submachine::NodeKey>,
    next_transition: Option<crate::diagram::model::transition::EdgeKey>,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
) {
    let (response, painter) =
        ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

    let zoom_delta = ui.input(|input| input.zoom_delta());
    if response.hovered() && zoom_delta != 1.0 {
        let old_zoom = viewport.camera_zoom.get();
        let new_zoom = Zoom::new(old_zoom * zoom_delta);
        if let Some(cursor) = ctx.input(|input| input.pointer.hover_pos()) {
            let cursor: LayoutPos = cursor.into();
            let world = (cursor - viewport.camera_pan) * (1.0 / old_zoom);
            viewport.camera_pan = cursor - world * new_zoom.get();
        }
        viewport.camera_zoom = new_zoom;
    }

    if response.dragged_by(egui::PointerButton::Primary) {
        viewport.camera_pan.x += response.drag_delta().x;
        viewport.camera_pan.y += response.drag_delta().y;
    }
    let scroll = ctx.input(|input| input.smooth_scroll_delta);
    if response.hovered() && scroll != egui::Vec2::ZERO {
        viewport.camera_pan.x += scroll.x;
        viewport.camera_pan.y += scroll.y;
    }

    let selection = SelectionState::default();
    render_grid(viewport, response.rect, &painter);
    render_edges(
        document.layout(),
        document.machine(),
        alphabet,
        viewport,
        &selection,
        next_transition,
        ui,
    );
    render_nodes(
        document.layout(),
        document.machine(),
        alphabet,
        viewport,
        &selection,
        executing_node,
        &painter,
    );
    render_notes(document.layout(), viewport, &selection, &painter);
}
