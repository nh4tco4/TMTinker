mod dispatch;
mod screen;
#[cfg(test)]
mod tests;
use crate::{
    commands::{Command, CommandId, REGISTRY},
    diagram::{
        compile::compile,
        editor::session::DiagramEditorSession,
        runtime::{DiagramRunSession, RunStatus, checks::BatchCheck},
        ui::{
            editor::{
                page::{EditorUiState, diagram_editor_page},
                tools_panel::tool_for_command,
            },
            runner::{RunnerUiState, diagram_runner_page},
        },
    },
    persistence::{
        self,
        transfer::{self, TransferEvent},
    },
    project::{
        Project,
        model::{ProjectEdit, TransactionKey},
    },
    ui::{
        menu::{MenuUiState, render_menu},
        theme::setup_visuals,
    },
};
use screen::{DiagramRunnerWorkspace, DiagramWorkspace, Screen};
use slotmap::{SlotMap, new_key_type};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};
new_key_type! { pub struct ProjectKey; }

pub struct App {
    cheatsheet_open: bool,
    projects: SlotMap<ProjectKey, Project>,
    screen: Screen,
    menu: MenuUiState,
    sender: Sender<TransferEvent>,
    receiver: Receiver<TransferEvent>,
    notice: Option<(String, bool)>,
    notice_started: Option<f64>,
    dirty: bool,
    save_requested: bool,
    next_save: f64,
    recovery: Option<String>,
}
impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(setup_visuals());
        cc.egui_ctx.style_mut(|style| {
            style.spacing.button_padding = egui::vec2(10., 7.);
            style.spacing.item_spacing = egui::vec2(8., 8.);
        });
        let (sender, receiver) = mpsc::channel();
        let mut projects = SlotMap::with_key();
        let mut notice = None;
        let mut recovery = None;
        if let Some(json) = cc
            .storage
            .and_then(|s| s.get_string(persistence::STORAGE_KEY))
        {
            match persistence::decode_library(&json) {
                Ok(saved) => {
                    for project in saved {
                        projects.insert(project);
                    }
                }
                Err(error) => {
                    notice = Some((error, true));
                    recovery = Some(json);
                }
            }
        }
        Self {
            cheatsheet_open: false,
            projects,
            screen: Screen::Menu,
            menu: MenuUiState::default(),
            sender,
            receiver,
            notice,
            notice_started: None,
            dirty: false,
            save_requested: false,
            next_save: 0.,
            recovery,
        }
    }
    fn project(&self) -> Option<&Project> {
        match &self.screen {
            Screen::Diagram(w) => self.projects.get(w.project),
            Screen::Menu => None,
        }
    }
    fn enabled(&self, id: CommandId) -> bool {
        use CommandId::*;
        let workspace = match &self.screen {
            Screen::Diagram(w) => Some(w),
            Screen::Menu => None,
        };
        let runner = workspace.and_then(|w| w.runner.as_ref());
        let checking = runner
            .and_then(|r| r.checks.as_ref())
            .is_some_and(|b| !b.is_done());
        match id {
            Cheatsheet => workspace.is_some(),
            NewProject | DeleteProject => workspace.is_none(),
            Import => true,
            OpenProject => true,
            Undo => runner.is_none() && self.project().is_some_and(Project::can_undo),
            Redo => runner.is_none() && self.project().is_some_and(Project::can_redo),
            Run | Step => {
                !checking && runner.is_some_and(|r| r.session.status() == RunStatus::Paused)
            }
            Pause => runner.is_some_and(|r| r.session.status() == RunStatus::Running),
            Reset | Stop | LoadTest | EditTape | EditTest | StepLimit => {
                !checking && runner.is_some()
            }
            CheckAll => {
                !checking
                    && runner.is_some()
                    && self
                        .project()
                        .is_some_and(|p| p.data().tests.iter().any(Option::is_some))
            }
            CancelChecks => checking,
            TapeLeftFive | TapeLeft | TapeRight | TapeRightFive | TapeHead => runner.is_some(),
            BackToEditor => !checking && runner.is_some(),
            OpenRunner | Validate | EditDiagram | AddMachine | RemoveMachine | RenameMachine
            | OpenMachine | PlaceMachine | MachineSettings => {
                workspace.is_some() && runner.is_none()
            }
            Move | Link | Note | Delete | Start | End | Left | Right | Write => {
                workspace.is_some() && runner.is_none()
            }
            Save | Export | Menu => workspace.is_some(),
        }
    }
    fn open(&mut self, key: ProjectKey) -> Result<(), String> {
        let project = self.projects.get(key).ok_or("Project no longer exists")?;
        self.screen = Screen::Diagram(Box::new(DiagramWorkspace {
            project: key,
            editor: DiagramEditorSession::new(project.document().entry_submachine()),
            ui: EditorUiState::default(),
            runner: None,
        }));
        Ok(())
    }
    fn persist(&mut self, storage: &mut dyn eframe::Storage) -> Result<(), String> {
        let json = persistence::encode_library(self.projects.values())?;
        if let Some(recovery) = &self.recovery {
            storage.set_string("tmtinker.recovery", recovery.clone());
            if storage.get_string("tmtinker.recovery").as_ref() != Some(recovery) {
                return Err(
                    "Cannot back up the previous project library. Export your project.".into(),
                );
            }
        }
        storage.set_string(persistence::STORAGE_KEY, json.clone());
        storage.flush();
        if storage.get_string(persistence::STORAGE_KEY).as_ref() != Some(&json) {
            return Err(
                "Cannot save projects. Storage may be full or unavailable. Export your project."
                    .into(),
            );
        }
        self.recovery = None;
        self.dirty = false;
        self.save_requested = false;
        Ok(())
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.layer_painter(egui::LayerId::background()).rect_filled(
            ctx.content_rect(),
            0.,
            crate::ui::theme::CANVAS_BACKGROUND,
        );
        self.process_transfers();
        self.show_notice(ctx);
        let now = ctx.input(|i| i.time);
        let mut commands = self.draw_screen(ctx, now);
        self.collect_shortcuts(ctx, &mut commands);
        for command in commands {
            if let Err(error) = self.dispatch(command, ctx) {
                self.set_notice((error, true));
            }
        }
        if ctx.input(|i| i.pointer.any_released())
            || (!ctx.wants_keyboard_input() && !ctx.input(|i| i.pointer.any_down()))
        {
            self.finish_gesture();
        }
        if self.cheatsheet_open {
            let runner = matches!(&self.screen, Screen::Diagram(w) if w.runner.is_some());
            crate::ui::cheatsheet::show(ctx, &mut self.cheatsheet_open, runner);
        }
        self.auto_save(ctx, frame, now);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Err(error) = self.persist(storage) {
            self.set_notice((error, true));
        }
    }
    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(2)
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [1., 1., 1., 1.]
    }
}

impl App {
    fn process_transfers(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                TransferEvent::Imported(result) => {
                    match result.and_then(|json| persistence::decode_project(&json)) {
                        Ok(project) => {
                            let key = self.projects.insert(project);
                            self.dirty = true;
                            self.save_requested = true;
                            if let Err(error) = self.open(key) {
                                self.set_notice((error, true));
                            }
                        }
                        Err(error) => self.set_notice((error, true)),
                    }
                }
                TransferEvent::Exported(result) => self.set_notice(match result {
                    Ok(()) => ("Project exported".into(), false),
                    Err(error) => (error, true),
                }),
            }
        }
    }
    fn set_notice(&mut self, notice: (String, bool)) {
        self.notice = Some(notice);
        self.notice_started = None;
    }
    fn show_notice(&mut self, ctx: &egui::Context) {
        let Some((message, error)) = self.notice.clone() else {
            return;
        };
        let now = ctx.input(|input| input.time);
        let started = *self.notice_started.get_or_insert(now);
        if now - started >= 5. {
            self.notice = None;
            self.notice_started = None;
            return;
        }
        ctx.request_repaint_after(Duration::from_secs_f64((5. - (now - started)).max(0.)));

        egui::Area::new(egui::Id::new("app_notice"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_TOP, [0., 10.])
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::WHITE)
                    .stroke(egui::Stroke::new(1., crate::ui::theme::PANEL_STROKE))
                    .corner_radius(6.)
                    .inner_margin(10.)
                    .show(ui, |ui| {
                        ui.set_max_width(520.);
                        ui.horizontal_wrapped(|ui| {
                            ui.colored_label(
                                if error {
                                    egui::Color32::DARK_RED
                                } else {
                                    egui::Color32::from_rgb(25, 125, 100)
                                },
                                message,
                            );
                            if ui.small_button("×").on_hover_text("Close").clicked() {
                                self.notice = None;
                            }
                        });
                    });
            });
    }
    fn draw_screen(&mut self, ctx: &egui::Context, now: f64) -> Vec<Command> {
        let commands = match &mut self.screen {
            Screen::Menu => render_menu(&self.projects, &mut self.menu, ctx),
            Screen::Diagram(w) => {
                let Some(project) = self.projects.get(w.project) else {
                    self.screen = Screen::Menu;
                    return Vec::new();
                };
                if let Some(runner) = &mut w.runner {
                    if runner.session.status() == RunStatus::Running {
                        if now >= runner.ui.next_step {
                            runner.session.step();
                            runner.ui.next_step = now + runner.ui.interval_ms as f64 / 1000.;
                        }
                        if runner.session.status() == RunStatus::Running {
                            ctx.request_repaint_after(Duration::from_secs_f64(
                                (runner.ui.next_step - now).max(0.001),
                            ));
                        }
                    }
                    if let Some(checks) = &mut runner.checks {
                        if !checks.is_done() {
                            checks.tick();
                            ctx.request_repaint();
                        }
                    }
                    diagram_runner_page(
                        project,
                        &runner.session,
                        &mut runner.ui,
                        runner.checks.as_ref(),
                        ctx,
                    )
                } else {
                    diagram_editor_page(project, &mut w.editor, &mut w.ui, ctx)
                }
            }
        };
        commands
    }
    fn collect_shortcuts(&self, ctx: &egui::Context, commands: &mut Vec<Command>) {
        let modal_open = self.cheatsheet_open
            || self.menu.delete_project.is_some()
            || self.menu.new_project_open
            || match &self.screen {
                Screen::Diagram(w) => {
                    w.ui.create_open
                        || w.ui.settings.is_some()
                        || w.runner.as_ref().is_some_and(|r| r.ui.has_modal())
                }
                Screen::Menu => false,
            };
        if !modal_open {
            if !ctx.wants_keyboard_input()
                && self.enabled(CommandId::Delete)
                && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete))
            {
                commands.push(Command::Invoke(CommandId::Delete));
            }
            let redo = egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::Z,
            );
            if self.enabled(CommandId::Redo) && ctx.input_mut(|i| i.consume_shortcut(&redo)) {
                commands.push(Command::Invoke(CommandId::Redo));
            }
            for info in REGISTRY {
                let Some(key) = info.key else {
                    continue;
                };
                if !self.enabled(info.id)
                    || (ctx.wants_keyboard_input()
                        && matches!(info.id, CommandId::Reset | CommandId::Stop))
                {
                    continue;
                }
                if info.modified {
                    let shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, key);
                    if ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                        commands.push(Command::Invoke(info.id));
                    }
                } else if !ctx.wants_keyboard_input()
                    && ctx.input(|i| i.modifiers.is_none() && i.key_pressed(key))
                {
                    commands.push(
                        tool_for_command(info.id)
                            .map_or(Command::Invoke(info.id), Command::SelectTool),
                    );
                }
            }
        }
    }
    fn finish_gesture(&mut self) {
        if let Screen::Diagram(w) = &self.screen {
            if let Some(project) = self.projects.get_mut(w.project) {
                project.finish_transaction();
            }
        }
    }
    fn auto_save(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, now: f64) {
        if self.save_requested || (self.dirty && now >= self.next_save) {
            if let Some(storage) = frame.storage_mut() {
                let explicit = self.save_requested;
                match self.persist(storage) {
                    Ok(()) => {
                        if explicit {
                            self.set_notice(("Projects saved".into(), false));
                        }
                    }
                    Err(error) => {
                        self.save_requested = false;
                        self.set_notice((error, true));
                    }
                }
            } else if self.save_requested {
                self.save_requested = false;
                self.set_notice((
                    "Storage unavailable. Export your project to keep it.".into(),
                    true,
                ));
            }
            self.next_save = now + 2.;
        }
        if self.dirty {
            ctx.request_repaint_after(Duration::from_secs_f64((self.next_save - now).max(0.1)));
        }
    }
}
