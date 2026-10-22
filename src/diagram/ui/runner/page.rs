use crate::{
    commands::Command,
    core::tape::Tape,
    diagram::{
        editor::session::{SelectionState, ViewportState, Zoom},
        model::{
            document::{DiagramDocument, SubmachineDocument},
            layout::LayoutPos,
            submachine::SubmachineKey,
        },
        runtime::{DiagramRunSession, checks::BatchCheck},
        ui::canvas::render::{
            render_edge::render_edges, render_grid::render_grid, render_node::render_nodes,
            render_note::render_notes,
        },
    },
    project::Project,
    shared::runner::{self, SharedRunnerUiState},
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
    pub shared: SharedRunnerUiState,
}
impl std::ops::Deref for RunnerUiState {
    type Target = SharedRunnerUiState;
    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}
impl std::ops::DerefMut for RunnerUiState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shared
    }
}
impl RunnerUiState {
    pub fn new(active: SubmachineKey, tape: &Tape) -> Self {
        Self {
            active_submachine: active,
            viewports: SecondaryMap::new(),
            canvas_top: 0.,
            canvas_rect: None,
            followed_step: None,
            shared: SharedRunnerUiState::new(tape),
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
    runner::controls_panel(
        project,
        session,
        &mut state.shared,
        checking,
        ctx,
        &mut commands,
        |ui, shared| {
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
                    shared.follow_execution = false;
                }
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
    runner_canvas(session, state, ctx);
    runner::test_dialog(project, session, &mut state.shared, ctx, &mut commands);
    commands
}
fn runner_canvas(session: &DiagramRunSession, state: &mut RunnerUiState, ctx: &egui::Context) {
    let location = session.source_location();
    if !state.follow_execution {
        state.followed_step = None;
    }
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
                    session.compiled().source(),
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

pub(crate) fn render_read_only_canvas(
    document: &SubmachineDocument,
    source: &DiagramDocument,
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
        source.alphabet(),
        viewport,
        &selection,
        next_transition,
        ui,
    );
    render_nodes(
        document.layout(),
        document.machine(),
        source,
        viewport,
        &selection,
        executing_node,
        &painter,
    );
    render_notes(document.layout(), viewport, &selection, &painter);
}
