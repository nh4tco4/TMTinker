use crate::{
    app::ProjectKey,
    diagram::{
        editor::session::DiagramEditorSession,
        runtime::{DiagramRunSession, checks::BatchCheck},
        ui::{editor::page::EditorUiState, runner::RunnerUiState},
    },
};
pub enum Screen {
    Menu,
    Diagram(Box<DiagramWorkspace>),
    Tu4(Box<Tu4Workspace>),
}
pub struct Tu4Workspace {
    pub project: ProjectKey,
    pub ui: crate::tu4::ui::Tu4UiState,
    pub runner: Option<Tu4RunnerWorkspace>,
}
pub struct Tu4RunnerWorkspace {
    pub session: crate::tu4::runtime::Tu4RunSession,
    pub checks: Option<crate::shared::checks::BatchCheck<crate::tu4::runtime::Tu4RunSession>>,
}
pub struct DiagramWorkspace {
    pub project: ProjectKey,
    pub editor: DiagramEditorSession,
    pub ui: EditorUiState,
    pub runner: Option<DiagramRunnerWorkspace>,
}
pub struct DiagramRunnerWorkspace {
    pub session: DiagramRunSession,
    pub ui: RunnerUiState,
    pub checks: Option<BatchCheck>,
}
