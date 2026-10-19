use slotmap::SecondaryMap;

pub use super::canvas_state::*;
use super::tool::EditorTool;
use crate::diagram::model::submachine::SubmachineKey;

pub struct DiagramEditorSession {
    active_submachine: SubmachineKey,
    tool: EditorTool,
    canvases: SecondaryMap<SubmachineKey, CanvasEditorState>,
    compile_error: Option<String>,
    compile_error_started: Option<f64>,
}

pub struct CanvasEditorState {
    pub viewport: ViewportState,
    pub interaction: CanvasInteractionState,
}

impl Default for CanvasEditorState {
    fn default() -> Self {
        Self {
            viewport: ViewportState::new(),
            interaction: CanvasInteractionState::default(),
        }
    }
}

impl DiagramEditorSession {
    pub fn new(active_submachine: SubmachineKey) -> Self {
        Self {
            active_submachine,
            tool: EditorTool::default(),
            canvases: SecondaryMap::new(),
            compile_error: None,
            compile_error_started: None,
        }
    }

    pub fn active_submachine(&self) -> SubmachineKey {
        self.active_submachine
    }

    pub fn open_submachine(&mut self, key: SubmachineKey) {
        self.active_submachine = key;
    }

    pub fn tool(&self) -> &EditorTool {
        &self.tool
    }

    pub fn clear_interactions(&mut self) {
        for (_, canvas) in self.canvases.iter_mut() {
            canvas.interaction = CanvasInteractionState::default();
        }
    }

    pub fn tool_mut(&mut self) -> &mut EditorTool {
        &mut self.tool
    }

    pub fn canvas_mut(&mut self, key: SubmachineKey) -> &mut CanvasEditorState {
        self.canvases.entry(key).unwrap().or_default()
    }

    pub fn compile_error(&self) -> Option<&str> {
        self.compile_error.as_deref()
    }

    pub fn set_compile_error(&mut self, message: String) {
        self.compile_error = Some(message);
        self.compile_error_started = None;
    }

    pub fn expire_compile_error(&mut self, ctx: &egui::Context) {
        if self.compile_error.is_some() {
            let now = ctx.input(|input| input.time);
            let started = *self.compile_error_started.get_or_insert(now);
            if now - started >= 5. {
                self.clear_compile_error();
            } else {
                ctx.request_repaint_after(std::time::Duration::from_secs_f64(5. - (now - started)));
            }
        }
    }

    pub fn clear_compile_error(&mut self) {
        self.compile_error = None;
        self.compile_error_started = None;
    }
}
