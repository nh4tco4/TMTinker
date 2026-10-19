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
