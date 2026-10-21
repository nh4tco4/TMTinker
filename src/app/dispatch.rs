use super::*;
use crate::core::tape::Tape;
use crate::diagram::model::submachine::SubmachineKey;

impl App {
    pub(super) fn dispatch(&mut self, command: Command, ctx: &egui::Context) -> Result<(), String> {
        if !self.enabled(command.id()) {
            return Ok(());
        }
        match command {
            Command::Create { name, alphabet } => {
                let key = self.projects.insert(Project::new(
                    name,
                    crate::diagram::model::document::DiagramDocument::new(alphabet),
                ));
                self.dirty = true;
                self.save_requested = true;
                self.open(key)?;
            }
            Command::CreateTu4 { name, alphabet } => {
                let key = self.projects.insert(Project::new_tu4(name, alphabet));
                self.dirty = true;
                self.save_requested = true;
                self.open(key)?;
            }
            Command::Open(key) => self.open(key)?,
            Command::DeleteProject(key) => {
                if self.projects.remove(key).is_some() {
                    self.dirty = true;
                    self.save_requested = true;
                }
            }
            Command::Edit { edit, group } => self.apply_edit(edit, group)?,
            Command::SelectTool(tool) => {
                if let Screen::Diagram(w) = &mut self.screen {
                    *w.editor.tool_mut() = tool;
                }
            }
            Command::OpenMachine(key) => self.open_machine(key)?,
            Command::LoadTest(slot) => {
                let tape = self
                    .project()
                    .and_then(|p| p.data().tests.get(slot))
                    .and_then(Option::as_ref)
                    .ok_or("Test is empty")?
                    .initial_tape
                    .clone();
                self.set_tape(tape)?;
            }
            Command::SetTape(tape) => self.set_tape(tape)?,
            Command::ScrollTape(delta) => {
                if let Screen::Tu4(w) = &mut self.screen {
                    w.ui.shared.tape_center = w.ui.shared.tape_center.saturating_add(delta);
                    w.ui.shared.follow_head = false;
                }
                if let Screen::Diagram(w) = &mut self.screen {
                    if let Some(runner) = &mut w.runner {
                        runner.ui.tape_center = runner.ui.tape_center.saturating_add(delta);
                        runner.ui.follow_head = false;
                    }
                }
            }
            Command::Invoke(id) => self.invoke(id, ctx)?,
        }
        Ok(())
    }

    fn apply_edit(
        &mut self,
        edit: ProjectEdit,
        group: Option<TransactionKey>,
    ) -> Result<(), String> {
        if let Screen::Tu4(w) = &mut self.screen {
            let project = self
                .projects
                .get_mut(w.project)
                .ok_or("Project no longer exists")?;
            if project.execute(edit, group)? {
                self.dirty = true;
                w.ui.error = None;
                if let Some(runner) = &mut w.runner {
                    runner
                        .session
                        .set_breakpoints(project.tu4().expect("TU4 project").breakpoints.clone());
                    runner.checks = None;
                }
            }
            return Ok(());
        }
        let removing_machine = matches!(&edit, ProjectEdit::RemoveMachine(_));
        let Screen::Diagram(w) = &mut self.screen else {
            return Ok(());
        };
        let project = self
            .projects
            .get_mut(w.project)
            .ok_or("Project no longer exists")?;
        if project.execute(edit, group)? {
            self.dirty = true;
            if removing_machine {
                if project
                    .document()
                    .submachine(w.editor.active_submachine())
                    .is_none()
                {
                    w.editor
                        .open_submachine(project.document().entry_submachine());
                }
                if matches!(w.editor.tool(), crate::diagram::editor::tool::EditorTool::Submachine(key)
                    if project.document().submachine(*key).is_none())
                {
                    *w.editor.tool_mut() = Default::default();
                }
                w.editor.clear_interactions();
                w.ui = EditorUiState::default();
            }
            if let Some(runner) = &mut w.runner {
                runner.checks = None;
            }
        }
        Ok(())
    }
    fn open_machine(&mut self, key: SubmachineKey) -> Result<(), String> {
        if let Screen::Diagram(w) = &mut self.screen {
            let project = self
                .projects
                .get(w.project)
                .ok_or("Project no longer exists")?;
            if project.document().submachine(key).is_some() {
                w.editor.open_submachine(key);
                w.ui = EditorUiState::default();
            }
        }
        Ok(())
    }
    fn set_tape(&mut self, tape: Tape) -> Result<(), String> {
        if let Screen::Tu4(w) = &mut self.screen {
            let project = self
                .projects
                .get_mut(w.project)
                .ok_or("Project no longer exists")?;
            if project.execute(
                ProjectEdit::SetTape(tape.clone()),
                Some(TransactionKey::PreparedTape),
            )? {
                self.dirty = true;
            }
            w.ui.shared.sync_tape(&tape);
            if let Some(runner) = &mut w.runner {
                runner.session.replace_tape(tape);
                runner.checks = None;
            }
            return Ok(());
        }
        let Screen::Diagram(w) = &mut self.screen else {
            return Ok(());
        };
        let project = self
            .projects
            .get_mut(w.project)
            .ok_or("Project no longer exists")?;
        if project.execute(
            ProjectEdit::SetTape(tape.clone()),
            Some(TransactionKey::PreparedTape),
        )? {
            self.dirty = true;
        }
        if let Some(runner) = &mut w.runner {
            runner.session.replace_tape(tape.clone());
            runner.ui.sync_tape(&tape);
            runner.checks = None;
        }
        Ok(())
    }
    fn invoke(&mut self, id: CommandId, ctx: &egui::Context) -> Result<(), String> {
        if id == CommandId::Delete {
            return self.delete_selection();
        }
        if let Some(tool) = tool_for_command(id) {
            return self.dispatch(Command::SelectTool(tool), ctx);
        }
        match id {
            CommandId::Cheatsheet => self.cheatsheet_open = true,
            CommandId::NewProject => self.menu.new_project_open = true,
            CommandId::AddMachine => {
                if let Screen::Diagram(w) = &mut self.screen {
                    w.ui.create_open = true;
                    w.ui.new_machine_name.clear();
                    w.ui.error = None;
                }
            }
            CommandId::Import => transfer::import(self.sender.clone(), ctx.clone()),
            CommandId::Export => {
                let project = self.project().ok_or("No open project")?;
                let json = persistence::encode_project(project)?;
                transfer::export(project.name(), json, self.sender.clone(), ctx.clone());
            }
            CommandId::Save => {
                self.save_requested = true;
                self.set_notice(("Saving projects…".into(), false));
            }
            CommandId::Menu => {
                self.finish_gesture();
                self.screen = Screen::Menu;
                self.save_requested = true;
            }
            CommandId::Undo | CommandId::Redo => self.change_history(id == CommandId::Undo)?,
            CommandId::OpenTu4 => {
                self.finish_gesture();
                let project = self.project().ok_or("No open project")?;
                let converted = crate::tu4::convert::convert(project.document())?;
                let mut data = project.data().clone();
                data.name = crate::project::model::ProjectName::new(format!(
                    "{} TU4",
                    project.name().chars().take(16).collect::<String>()
                ))
                .map_err(|e| e.to_string())?;
                data.document = crate::project::model::ProjectDocument::Tu4(converted);
                data.created_at = crate::project::model::timestamp();
                let key = self.projects.insert(Project::from_data(data));
                self.dirty = true;
                self.save_requested = true;
                self.open(key)?;
            }
            CommandId::OpenRunner | CommandId::Validate => {
                self.compile_project(id == CommandId::OpenRunner)?
            }
            _ => self.runner_command(id, ctx)?,
        }
        Ok(())
    }
    fn delete_selection(&mut self) -> Result<(), String> {
        let Screen::Diagram(w) = &mut self.screen else {
            return Ok(());
        };
        let active = w.editor.active_submachine();
        let selected = w
            .editor
            .canvas_mut(active)
            .interaction
            .selection_state
            .selected
            .clone();
        if selected.is_empty() {
            return Ok(());
        }
        let project = self
            .projects
            .get(w.project)
            .ok_or("Project no longer exists")?;
        let Some(original) = project.document().submachine(active) else {
            return Ok(());
        };
        if original.is_system() {
            return Ok(());
        }
        let mut machine = original.clone();
        for item in selected {
            crate::diagram::editor::edit_tools::delete_tool(&mut machine, item);
        }
        w.editor.clear_interactions();
        w.ui.nodes = Default::default();
        w.ui.edges = Default::default();
        self.finish_gesture();
        self.apply_edit(
            ProjectEdit::ReplaceMachine {
                key: active,
                machine: Box::new(machine),
            },
            None,
        )
    }

    fn change_history(&mut self, undo: bool) -> Result<(), String> {
        if let Screen::Tu4(w) = &mut self.screen {
            let project = self
                .projects
                .get_mut(w.project)
                .ok_or("Project no longer exists")?;
            if if undo { project.undo() } else { project.redo() } {
                self.dirty = true;
            }
            w.ui.error = None;
            return Ok(());
        }
        let Screen::Diagram(w) = &mut self.screen else {
            return Ok(());
        };
        let project = self
            .projects
            .get_mut(w.project)
            .ok_or("Project no longer exists")?;
        let changed = if undo { project.undo() } else { project.redo() };
        if !changed {
            return Ok(());
        }
        self.dirty = true;
        if project
            .document()
            .submachine(w.editor.active_submachine())
            .is_none()
        {
            w.editor
                .open_submachine(project.document().entry_submachine());
        }
        w.editor.clear_interactions();
        if matches!(w.editor.tool(), crate::diagram::editor::tool::EditorTool::Submachine(key)
            if project.document().submachine(*key).is_none())
        {
            *w.editor.tool_mut() = Default::default();
        }
        w.ui = EditorUiState::default();
        if w.runner.is_some() {
            match make_runner(project) {
                Ok(runner) => w.runner = Some(runner),
                Err(error) => {
                    w.runner = None;
                    w.editor.set_compile_error(error);
                }
            }
        }
        Ok(())
    }
    fn compile_project(&mut self, run: bool) -> Result<(), String> {
        if let Screen::Tu4(w) = &mut self.screen {
            let project = self
                .projects
                .get_mut(w.project)
                .ok_or("Project no longer exists")?;
            project.finish_transaction();
            let document = project.tu4().ok_or("Expected a TU4 project")?;
            match crate::tu4::compile::compile(&document.source, &document.alphabet) {
                Ok(compiled) => {
                    w.ui.error = None;
                    if run {
                        w.ui.shared.sync_tape(&project.data().prepared_tape);
                        w.runner = Some(Tu4RunnerWorkspace {
                            session: {
                                let mut session = crate::tu4::runtime::Tu4RunSession::new(
                                    compiled,
                                    project.data().prepared_tape.clone(),
                                );
                                session.set_breakpoints(document.breakpoints.clone());
                                session
                            },
                            checks: None,
                        });
                    } else {
                        self.set_notice(("TU4 program is valid".into(), false));
                    }
                }
                Err(error) => w.ui.error = Some(error.to_string()),
            }
            return Ok(());
        }
        let Screen::Diagram(w) = &mut self.screen else {
            return Ok(());
        };
        let project = self
            .projects
            .get_mut(w.project)
            .ok_or("Project no longer exists")?;
        project.finish_transaction();
        if run {
            match make_runner(project) {
                Ok(runner) => {
                    w.editor.clear_compile_error();
                    w.runner = Some(runner);
                }
                Err(error) => w.editor.set_compile_error(error),
            }
        } else {
            match compile(project.document()) {
                Ok(_) => {
                    w.editor.clear_compile_error();
                    self.set_notice(("Diagram is valid".into(), false));
                }
                Err(error) => w.editor.set_compile_error(error.to_string()),
            }
        }
        Ok(())
    }
    fn runner_command(&mut self, id: CommandId, ctx: &egui::Context) -> Result<(), String> {
        match &mut self.screen {
            Screen::Tu4(w) => {
                if id == CommandId::BackToEditor {
                    w.runner = None;
                    return Ok(());
                }
                if let Some(runner) = &mut w.runner {
                    let project = self
                        .projects
                        .get(w.project)
                        .ok_or("Project no longer exists")?;
                    runner_command(
                        id,
                        &mut runner.session,
                        &mut w.ui.shared,
                        &mut runner.checks,
                        project,
                        ctx,
                    );
                }
            }
            Screen::Diagram(w) => {
                if id == CommandId::BackToEditor {
                    w.runner = None;
                    return Ok(());
                }
                if let Some(runner) = &mut w.runner {
                    let project = self
                        .projects
                        .get(w.project)
                        .ok_or("Project no longer exists")?;
                    runner_command(
                        id,
                        &mut runner.session,
                        &mut runner.ui.shared,
                        &mut runner.checks,
                        project,
                        ctx,
                    );
                }
            }
            Screen::Menu => {}
        }
        Ok(())
    }
}

fn runner_command<S: RunSession>(
    id: CommandId,
    session: &mut S,
    ui: &mut crate::shared::runner::SharedRunnerUiState,
    checks: &mut Option<crate::shared::checks::BatchCheck<S>>,
    project: &Project,
    ctx: &egui::Context,
) {
    match id {
        CommandId::Run => {
            session.run();
            ui.next_step = ctx.input(|i| i.time) + ui.interval_ms as f64 / 1000.;
            ctx.request_repaint();
        }
        CommandId::Pause => session.pause(),
        CommandId::Step => {
            session.pause();
            session.step();
        }
        CommandId::Reset => {
            session.reset();
            *checks = None;
        }
        CommandId::Stop => session.stop(),
        CommandId::TapeHead => {
            ui.follow_head = true;
            ui.tape_center = session.tape().head_position();
        }
        CommandId::CheckAll => {
            session.pause();
            *checks = Some(crate::shared::checks::BatchCheck::new(
                session.program(),
                project.data().tests.clone(),
                project.data().step_limit,
            ));
            ctx.request_repaint();
        }
        CommandId::CancelChecks => *checks = None,
        _ => {}
    }
}

fn make_runner(project: &Project) -> Result<DiagramRunnerWorkspace, String> {
    let compiled = compile(project.document()).map_err(|e| e.to_string())?;
    let tape = project.data().prepared_tape.clone();
    let active = compiled.source().entry_submachine();
    Ok(DiagramRunnerWorkspace {
        session: DiagramRunSession::new(compiled, tape.clone()),
        ui: RunnerUiState::new(active, &tape),
        checks: None,
    })
}
