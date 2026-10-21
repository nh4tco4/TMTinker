use std::{ops::Range, sync::Arc};

use super::runtime::Tu4RunSession;
use crate::{
    commands::Command,
    project::{
        Project,
        model::{ProjectEdit, TransactionKey},
    },
    shared::{
        checks::BatchCheck,
        runner::{self, SharedRunnerUiState},
        runtime::RunSession as _,
    },
};

const FOLLOW_LINE_LIMIT: usize = 2500;
const PAGE_LINES: usize = 1000;

pub struct Tu4UiState {
    pub shared: SharedRunnerUiState,
    pub error: Option<String>,
    editor: CodeEditor,
    followed: Option<(u64, Option<usize>)>,
}

impl Tu4UiState {
    pub fn new(project: &Project) -> Self {
        Self {
            shared: SharedRunnerUiState::new(&project.data().prepared_tape),
            error: None,
            editor: CodeEditor::new(
                project
                    .tu4()
                    .expect("TU4 workspace has a TU4 document")
                    .source
                    .clone(),
            ),
            followed: None,
        }
    }
}

struct CodeEditor {
    source: Arc<str>,
    starts: Vec<usize>,
    page: usize,
    range: Range<usize>,
    text: String,
    revision: usize,
}

impl CodeEditor {
    fn new(source: Arc<str>) -> Self {
        let mut editor = Self {
            source,
            starts: Vec::new(),
            page: 0,
            range: 0..0,
            text: String::new(),
            revision: 0,
        };
        editor.index();
        editor.load_page();
        editor
    }
    fn index(&mut self) {
        self.starts = std::iter::once(0)
            .chain(
                self.source
                    .match_indices('\n')
                    .map(|(offset, _)| offset + 1)
                    .filter(|&offset| offset < self.source.len()),
            )
            .collect();
    }
    fn page_size(&self) -> usize {
        if self.starts.len() <= FOLLOW_LINE_LIMIT {
            FOLLOW_LINE_LIMIT
        } else {
            PAGE_LINES
        }
    }
    fn load_page(&mut self) {
        let size = self.page_size();
        self.page = self.page.min((self.starts.len() - 1) / size);
        let first = self.page * size;
        self.range = *self
            .starts
            .get(first)
            .expect("page begins at a source line")
            ..self
                .starts
                .get(first + size)
                .copied()
                .unwrap_or(self.source.len());
        self.text = self
            .source
            .get(self.range.clone())
            .expect("page boundaries are UTF-8 offsets")
            .to_owned();
    }
    fn change_page(&mut self, page: usize) {
        self.page = page;
        self.revision = self.revision.wrapping_add(1);
        self.load_page();
    }
    fn sync(&mut self, source: &Arc<str>) {
        if !Arc::ptr_eq(&self.source, source) {
            self.source = source.clone();
            self.revision = self.revision.wrapping_add(1);
            self.index();
            self.load_page();
        }
    }
    fn edit(&mut self, ui: &mut egui::Ui) -> Option<Arc<str>> {
        if self.starts.len() > FOLLOW_LINE_LIMIT {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.page > 0, egui::Button::new("←"))
                    .clicked()
                {
                    self.change_page(self.page - 1);
                }
                let mut page = self.page + 1;
                let pages = self.starts.len().div_ceil(PAGE_LINES);
                ui.label("Page");
                if ui
                    .add(egui::DragValue::new(&mut page).range(1..=pages))
                    .changed()
                {
                    self.change_page(page - 1);
                }
                ui.label(format!(
                    "Lines {}–{} of {}",
                    self.page * PAGE_LINES + 1,
                    ((self.page + 1) * PAGE_LINES).min(self.starts.len()),
                    self.starts.len()
                ));
                if ui
                    .add_enabled(
                        (self.page + 1) * PAGE_LINES < self.starts.len(),
                        egui::Button::new("→"),
                    )
                    .clicked()
                {
                    self.change_page(self.page + 1);
                }
            });
        }
        let response = egui::ScrollArea::both()
            .id_salt("tu4_edit_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.text)
                        .id_salt(("tu4_source", self.page, self.revision))
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .hint_text("1,λ,>,2\n2,λ,~,0\n// state, symbol, action, next state"),
                )
            })
            .inner;
        if !response.changed() {
            return None;
        }
        let mut source =
            String::with_capacity(self.source.len() - self.range.len() + self.text.len());
        source.push_str(
            self.source
                .get(..self.range.start)
                .expect("page begins at a UTF-8 boundary"),
        );
        source.push_str(&self.text);
        source.push_str(
            self.source
                .get(self.range.end..)
                .expect("page ends at a UTF-8 boundary"),
        );
        self.source = Arc::from(source);
        self.index();
        self.load_page();
        Some(self.source.clone())
    }
    fn line(&self, index: usize) -> &str {
        let end = self
            .starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.source.len());
        self.source
            .get(*self.starts.get(index).expect("visible line exists")..end)
            .expect("line boundaries are UTF-8 offsets")
            .trim_end_matches(['\r', '\n'])
    }
}

fn breakpoint_button(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    line: usize,
    selected: bool,
    commands: &mut Vec<Command>,
) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let response = ui.interact(
        rect,
        ui.id().with(("breakpoint", line)),
        egui::Sense::click(),
    );
    if selected {
        ui.painter()
            .circle_filled(rect.center(), 5., egui::Color32::from_rgb(210, 50, 50));
    } else {
        ui.painter().circle_stroke(
            rect.center(),
            5.,
            egui::Stroke::new(1., egui::Color32::GRAY),
        );
    }
    if response
        .on_hover_text(if selected {
            "Remove breakpoint"
        } else {
            "Set breakpoint"
        })
        .clicked()
    {
        commands.push(Command::Edit {
            edit: ProjectEdit::ToggleBreakpoint(line),
            group: None,
        });
    }
}

pub fn editor_page(project: &Project, state: &mut Tu4UiState, ctx: &egui::Context) -> Vec<Command> {
    let mut commands = Vec::new();
    state
        .editor
        .sync(&project.tu4().expect("TU4 editor has a TU4 document").source);
    egui::SidePanel::right("tu4_controls")
        .exact_width(216.)
        .resizable(false)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            crate::ui::theme::sidebar(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("tu4_editor_controls")
                    .show(ui, |ui| {
                        crate::shared::project_controls::control_panel(project, ui, &mut commands);
                    });
            });
        });
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("TU4");
        ui.monospace("state, symbol, action, next state");
        ui.weak("< left · > right · ~ halt · λ or space for blank · // comment");
        if let Some(error) = &state.error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
        ui.separator();
        if let Some(source) = state.editor.edit(ui) {
            state.error = None;
            commands.push(Command::Edit {
                edit: ProjectEdit::SetSource(source),
                group: Some(TransactionKey::Source),
            });
        }
    });
    commands
}

pub fn runner_page(
    project: &Project,
    session: &Tu4RunSession,
    state: &mut Tu4UiState,
    batch: Option<&BatchCheck<Tu4RunSession>>,
    ctx: &egui::Context,
) -> Vec<Command> {
    let mut commands = Vec::new();
    state
        .editor
        .sync(&project.tu4().expect("TU4 runner has a TU4 document").source);
    let checking = batch.is_some_and(|batch| !batch.is_done());
    runner::controls_panel(
        project,
        session,
        &mut state.shared,
        checking,
        ctx,
        &mut commands,
        |ui, _| {
            ui.label(format!("State: {}", session.state_number()));
            if let Some(line) = session.line() {
                ui.label(format!("Line: {line}"));
            }
        },
    );
    runner::tests_panel(project, &mut state.shared, batch, ctx, &mut commands);
    runner::tape_panel(
        project,
        session,
        &mut state.shared,
        checking,
        ctx,
        &mut commands,
    );
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("TU4");
        let highlight = state.editor.starts.len() <= FOLLOW_LINE_LIMIT;
        if !highlight {
            ui.weak("Execution line highlighting is available up to 2500 lines.");
        }
        let current = session.line();
        let key = (session.steps(), current);
        let mut scroll = egui::ScrollArea::both()
            .id_salt("tu4_run_scroll")
            .auto_shrink([false, false]);
        let row_height = ui
            .text_style_height(&egui::TextStyle::Monospace)
            .max(ui.spacing().interact_size.y);
        let row_stride = row_height + ui.spacing().item_spacing.y;
        let padding_rows = (ui.available_height() / row_stride).ceil() as usize;
        if highlight && state.shared.follow_execution {
            if state.followed != Some(key) {
                if let Some(line) = current {
                    let offset = (line.saturating_sub(1) as f32 * row_stride
                        - ui.available_height() * 0.4)
                        .max(0.);
                    scroll = scroll.vertical_scroll_offset(offset);
                }
                state.followed = Some(key);
            }
        } else {
            state.followed = None;
        }
        scroll.show_rows(
            ui,
            row_height,
            state.editor.starts.len() + padding_rows,
            |ui, rows| {
                for index in rows {
                    if index >= state.editor.starts.len() {
                        ui.allocate_space(egui::vec2(1., row_height));
                        continue;
                    }
                    let active = highlight && current == Some(index + 1);
                    egui::Frame::new()
                        .fill(if active {
                            egui::Color32::from_rgb(215, 241, 225)
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(20., row_height),
                                    egui::Sense::hover(),
                                );
                                let text = state.editor.line(index);
                                let code = text.split_once("//").map_or(text, |(code, _)| code);
                                if !code.trim().is_empty() {
                                    breakpoint_button(
                                        ui,
                                        rect,
                                        index + 1,
                                        project
                                            .tu4()
                                            .expect("TU4 project")
                                            .breakpoints
                                            .contains(&(index + 1)),
                                        &mut commands,
                                    );
                                }
                                ui.add_sized(
                                    [60., row_height],
                                    egui::Label::new(
                                        egui::RichText::new((index + 1).to_string())
                                            .monospace()
                                            .weak(),
                                    ),
                                );
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(state.editor.line(index)).monospace(),
                                    )
                                    .extend()
                                    .selectable(false),
                                );
                            });
                        });
                }
            },
        );
    });
    runner::test_dialog(project, session, &mut state.shared, ctx, &mut commands);
    commands
}

pub fn cheatsheet(ctx: &egui::Context, open: &mut bool) {
    let response = egui::Modal::new(egui::Id::new("tu4_cheatsheet")).show(ctx, |ui| {
        ui.set_width((ctx.content_rect().width() - 80.).clamp(120., 520.));
        ui.heading("TU4 cheatsheet");
        egui::ScrollArea::vertical().max_height((ctx.content_rect().height() - 150.).max(80.)).show(ui, |ui| {
        ui.label("Write one command per line:");
        ui.monospace("state, symbol, action, next state");
        ui.monospace("1,λ,>,2\n2,λ,1,3\n3,1,~,0");
        ui.label("Use < to move left, > to move right, and ~ to halt. Write an alphabet symbol to replace the current cell. Use λ or a space for blank, and // for comments.");
        ui.label("State numbers range from 0 to 4294967295. Execution starts at the state of the first command. Halt ignores the fourth field. Writing the same symbol in the same state continues execution.");
        ui.separator();
        ui.label("Run starts execution; Pause suspends it. Step executes one command. Reset restores the initial tape and state; Stop ends execution. Editor unlocks the source text.");
        ui.label("Click the circle beside a command to toggle a breakpoint. Execution pauses before that command; Run continues past the current stop and pauses at the next breakpoint. Step crosses a breakpoint. Breakpoints are saved with the project.");
        ui.label("Run without delay executes commands in batches. Programs up to 2500 lines highlight the current command; Follow step execution scrolls to it. Larger programs use pages in the editor.");
        ui.label("Load a test to use its initial tape. The gear edits its initial and expected tapes. Check all six compares results relative to the head, with a step limit per test.");
        ui.label("Tape notation: mark the head with parentheses, for example 01(1)0 or (λ). Save stores the project library; Export saves the project as JSON, including the source, tape and tests.");
        });
        if ui.button("Close · Esc").clicked() { *open = false; }
    });
    if response.should_close() {
        *open = false;
    }
}
