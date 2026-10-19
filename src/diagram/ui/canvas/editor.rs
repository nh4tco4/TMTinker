use egui::{Pos2, Rect, Vec2};

use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::{
        editor::{
            basic_node_tools::{
                left_tool, right_tool, start_tool, stop_tool, submachine_tool, write_tool,
            },
            edit_tools::{delete_tool, link_tool, note_tool},
            session::{
                CanvasEditorState, CanvasInteractionState, RectSelection, SelectableKey,
                ViewportState, Zoom,
            },
            tool::{BasicActionTools, EditTools, EditorTool},
        },
        model::{
            document::SubmachineDocument,
            layout::{GraphLayout, LayoutPos},
            submachine::Submachine,
        },
        ui::{
            canvas::render::{
                render_edge::{build_edge_points, edge_label_layout, render_edges},
                render_grid::render_grid,
                render_node::render_nodes,
                render_note::{note_rect, render_notes},
            },
            editor::{edge_inspector::edge_options_base, node_inspector::node_options_base},
        },
    },
    ui::theme::{NODE_SIZE, SELECTED_NODE_FILL},
};

pub fn handle_canvas(
    submachine_document: &mut SubmachineDocument,
    canvas_state: &mut CanvasEditorState,
    current_tool: &EditorTool,
    alphabet: ValidatedAlphabet,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    node_state: &mut crate::diagram::ui::editor::node_inspector::NodeInspectorState,
    edge_state: &mut crate::diagram::ui::editor::edge_inspector::EdgeInspectorState,
) {
    let CanvasEditorState {
        viewport,
        interaction: canvas_interaction_state,
    } = canvas_state;
    let (resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    let rect = resp.rect;

    if !ctx.wants_keyboard_input() && ctx.memory(|m| m.top_modal_layer().is_none()) {
        for (key, delta) in [
            (
                egui::Key::ArrowLeft,
                LayoutPos::new(-GraphLayout::GRID_SIZE, 0.),
            ),
            (
                egui::Key::ArrowRight,
                LayoutPos::new(GraphLayout::GRID_SIZE, 0.),
            ),
            (
                egui::Key::ArrowUp,
                LayoutPos::new(0., -GraphLayout::GRID_SIZE),
            ),
            (
                egui::Key::ArrowDown,
                LayoutPos::new(0., GraphLayout::GRID_SIZE),
            ),
        ] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                for node in canvas_interaction_state
                    .selection_state
                    .selected_node_keys()
                {
                    if let Some(position) = submachine_document.layout().get_node_position(node) {
                        submachine_document
                            .layout_mut()
                            .set_node_position(node, position + delta);
                    }
                }
            }
        }
    }

    // Handle zoom
    //  TODO turn it into function
    {
        let zoom_delta = ui.input(|i| i.zoom_delta());
        if zoom_delta != 1.0 {
            let old_zoom = viewport.camera_zoom.get();
            let new_zoom = Zoom::new(viewport.camera_zoom.get() * zoom_delta);

            if let Some(cursor_screen) = ctx.input(|i| i.pointer.hover_pos()) {
                let cursor_screen: LayoutPos = cursor_screen.into();
                let cursor_world: LayoutPos =
                    (cursor_screen - viewport.camera_pan) * (1.0 / old_zoom);

                let new_pan: LayoutPos = cursor_screen - cursor_world * new_zoom.get();

                viewport.camera_zoom = new_zoom;
                viewport.camera_pan = new_pan;
            } else {
                viewport.camera_zoom = new_zoom;
            }
        }
    }

    if let Some(pointer_position) = ctx.input(|i| i.pointer.hover_pos()) {
        let hovered_key = compute_hovered_key(
            submachine_document.layout(),
            viewport,
            submachine_document.machine(),
            &alphabet,
            &rect,
            pointer_position,
            &painter,
        );
        canvas_interaction_state.selection_state.hovered = hovered_key;

        // Handle canvas drag with LMB
        //  TODO turn it into function
        {
            if resp.drag_started_by(egui::PointerButton::Primary) {
                let pressed_key =
                    ctx.input(|input| input.pointer.press_origin())
                        .and_then(|position| {
                            compute_hovered_key(
                                submachine_document.layout(),
                                viewport,
                                submachine_document.machine(),
                                &alphabet,
                                &rect,
                                position,
                                &painter,
                            )
                        });
                match pressed_key {
                    Some(SelectableKey::Node(key)) => {
                        let item = SelectableKey::Node(key);

                        if !canvas_interaction_state.selection_state.is_selected(&item) {
                            if !ctx.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                                canvas_interaction_state.selection_state.clear();
                            }

                            canvas_interaction_state.selection_state.single_select(item);
                        }

                        canvas_interaction_state.drag =
                            canvas_interaction_state.selection_state.selected.clone();
                    }
                    Some(SelectableKey::Note(key)) => {
                        let item = SelectableKey::Note(key);

                        if !canvas_interaction_state.selection_state.is_selected(&item) {
                            if !ctx.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                                canvas_interaction_state.selection_state.clear();
                            }

                            canvas_interaction_state.selection_state.single_select(item);
                        }

                        canvas_interaction_state.drag = vec![item];
                    }
                    _ => {
                        canvas_interaction_state
                            .drag
                            .drain(..)
                            .for_each(|i| match i {
                                SelectableKey::Node(key) => {
                                    submachine_document.layout_mut().update_node_position(key)
                                }
                                SelectableKey::Note(key) => {
                                    submachine_document.layout_mut().update_note_position(key)
                                }
                                _ => (),
                            });
                    }
                }
            }
        }

        // Handle rectangle selection
        // TODO turn it into function
        {
            if resp.drag_started_by(egui::PointerButton::Secondary) {
                canvas_interaction_state.rect_select = Some(RectSelection {
                    start: pointer_position,
                    current: pointer_position,
                });
                if !ctx.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                    canvas_interaction_state.selection_state.clear();
                }
            }

            if resp.dragged_by(egui::PointerButton::Secondary) {
                if let Some(rectangle) = canvas_interaction_state.rect_select.as_mut() {
                    rectangle.current = pointer_position;
                }
            }

            if resp.drag_stopped_by(egui::PointerButton::Secondary) {
                if let Some(rectangle) = canvas_interaction_state.rect_select.take() {
                    let select_rect = egui::Rect::from_two_pos(rectangle.start, rectangle.current);

                    let selected =
                        select_items_in_rect(submachine_document.layout(), viewport, select_rect);

                    canvas_interaction_state.selection_state.selected = selected;
                }
            }
        }

        // Handle click
        // TODO turn it into function
        {
            if resp.clicked() {
                let modifiers = ctx.input(|i| i.modifiers);

                match hovered_key {
                    Some(id) => {
                        if modifiers.shift || modifiers.command {
                            canvas_interaction_state.selection_state.toggle_selected(id);
                        } else {
                            canvas_interaction_state.selection_state.single_select(id);
                        }
                    }

                    None => {
                        if !modifiers.ctrl && !modifiers.shift && !modifiers.command {
                            canvas_interaction_state.selection_state.clear();
                        }
                    }
                }

                handle_click(
                    submachine_document,
                    viewport,
                    current_tool,
                    pointer_position,
                    canvas_interaction_state,
                    &alphabet,
                );
            }
        }

        if let Some(node_key) = canvas_interaction_state.selection_state.single_node()
            && *current_tool != EditorTool::Edit(EditTools::Link)
            && *current_tool != EditorTool::Edit(EditTools::Delete)
        {
            if let Some(node_position) = submachine_document.layout().get_node_position(node_key) {
                let window_pos = viewport.world_to_screen(node_position);

                node_options_base(
                    viewport,
                    submachine_document.machine_mut(),
                    &alphabet,
                    ctx,
                    node_key,
                    window_pos.into(),
                    node_state,
                );
            }
        }

        if let Some(edge_key) = canvas_interaction_state.selection_state.single_edge()
            && *current_tool != EditorTool::Edit(EditTools::Delete)
        {
            let window_pos = pointer_position;

            edge_options_base(
                viewport,
                submachine_document.machine_mut(),
                &alphabet,
                ctx,
                edge_key,
                window_pos.into(),
                edge_state,
            );
        }
    }

    if let [SelectableKey::Note(key)] = canvas_interaction_state.selection_state.selected.as_slice()
        && *current_tool != EditorTool::Edit(EditTools::Delete)
        && let Some(note) = submachine_document.layout_mut().note_mut(*key)
    {
        egui::Window::new("Note")
            .id(egui::Id::new(("note_editor", key)))
            .resizable(false)
            .default_width(280.)
            .show(ctx, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut note.content)
                        .desired_rows(4)
                        .desired_width(260.),
                );
            });
    }

    // TODO add different cursor variants
    // if app.rect_select_start.is_some() {
    //     ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
    // } else if editor.selected.hovered.is_some() {
    //     ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    // } else if resp.dragged_by(egui::PointerButton::Primary) || !app.dragged_nodes.is_empty() {
    //     ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
    // } else {
    //     ctx.set_cursor_icon(egui::CursorIcon::Default);
    // }

    // Handle scroll
    // TODO turn it into function
    {
        let scroll = ctx.input(|i| i.smooth_scroll_delta);
        let modifiers = ctx.input(|i| i.modifiers);
        if scroll != egui::Vec2::ZERO && !modifiers.ctrl && resp.hovered() {
            // egui already maps Shift + wheel to the horizontal axis.
            viewport.camera_pan.x += scroll.x;
            viewport.camera_pan.y += scroll.y;
        }
    }

    // Dragging
    // TODO turn it into function
    {
        if resp.dragged_by(egui::PointerButton::Primary) {
            if !canvas_interaction_state.drag.is_empty() {
                let zoom = viewport.camera_zoom.get();
                let delta = resp.drag_delta();

                let delta = LayoutPos {
                    x: delta.x / zoom,
                    y: delta.y / zoom,
                };

                for item in &canvas_interaction_state.drag {
                    match item {
                        SelectableKey::Node(key) => {
                            submachine_document.move_node_unsnapped(*key, delta);
                        }

                        SelectableKey::Note(key) => {
                            submachine_document
                                .layout_mut()
                                .move_note_unsnapped(*key, delta);
                        }

                        SelectableKey::Edge(_) => {}
                    }
                }
            } else {
                viewport.camera_pan.x += resp.drag_delta().x;
                viewport.camera_pan.y += resp.drag_delta().y;
            }
        }

        if resp.drag_stopped_by(egui::PointerButton::Primary) {
            for item in canvas_interaction_state.drag.drain(..) {
                match item {
                    SelectableKey::Node(key) => {
                        submachine_document.layout_mut().update_node_position(key);
                    }

                    SelectableKey::Note(key) => {
                        submachine_document.layout_mut().update_note_position(key);
                    }

                    SelectableKey::Edge(_) => {}
                }
            }
        }
    }

    {
        render_grid(viewport, rect, &painter);

        if let Some(rectangle) = canvas_interaction_state.rect_select.as_ref() {
            render_selection_rect(&painter, rectangle.start, rectangle.current);
        }

        render_edges(
            submachine_document.layout(),
            submachine_document.machine(),
            &alphabet,
            viewport,
            &canvas_interaction_state.selection_state,
            None,
            ui,
        );

        render_drag_snap_preview(
            &painter,
            submachine_document,
            viewport,
            canvas_interaction_state,
        );

        render_nodes(
            submachine_document.layout(),
            submachine_document.machine(),
            &alphabet,
            viewport,
            &canvas_interaction_state.selection_state,
            None,
            &painter,
        );

        render_notes(
            submachine_document.layout(),
            viewport,
            &canvas_interaction_state.selection_state,
            &painter,
        );
    }
}

fn handle_click(
    submachine_document: &mut SubmachineDocument,
    viewport: &ViewportState,
    current_tool: &EditorTool,
    pointer_position: egui::Pos2,
    canvas_interaction_state: &mut CanvasInteractionState,
    alphabet: &ValidatedAlphabet,
) {
    let world = viewport.screen_to_world(pointer_position.into());

    let snapped = GraphLayout::snap_to_grid(world).into();

    match current_tool {
        EditorTool::Place(BasicActionTools::Left) => {
            left_tool(submachine_document, snapped);
        }
        EditorTool::Place(BasicActionTools::Right) => {
            right_tool(submachine_document, snapped);
        }
        EditorTool::Place(BasicActionTools::Write) => {
            write_tool(submachine_document, snapped);
        }
        EditorTool::Place(BasicActionTools::Start) => {
            start_tool(submachine_document, snapped);
        }
        EditorTool::Place(BasicActionTools::Stop) => {
            stop_tool(submachine_document, snapped);
        }
        EditorTool::Edit(EditTools::Comment) => {
            let key = note_tool(submachine_document, snapped.into());
            canvas_interaction_state
                .selection_state
                .single_select(SelectableKey::Note(key));
        }
        EditorTool::Edit(EditTools::Delete) => {
            if let Some(key) = canvas_interaction_state.selection_state.hovered {
                delete_tool(submachine_document, key);
                canvas_interaction_state.selection_state.clear();
            }
        }
        EditorTool::Edit(EditTools::Link) => {
            if canvas_interaction_state.selection_state.selected.len() == 1 {
                link_tool(
                    submachine_document,
                    &mut canvas_interaction_state.link_source,
                    alphabet,
                    canvas_interaction_state
                        .selection_state
                        .selected
                        .get(0)
                        .copied(),
                );
            }
        }
        EditorTool::Edit(EditTools::Move) => {}
        EditorTool::Submachine(key) => {
            submachine_tool(submachine_document, snapped, key);
        }
    }
}

fn distance_to_segment(p: egui::Pos2, v: egui::Pos2, w: egui::Pos2) -> f32 {
    let l2 = v.distance_sq(w);
    if l2 == 0.0 {
        return p.distance(v);
    }
    let t = ((p - v).dot(w - v) / l2).clamp(0.0, 1.0);
    p.distance(v + (w - v) * t)
}

fn compute_hovered_key(
    graph: &GraphLayout,
    viewport: &ViewportState,
    submachine: &Submachine,
    alphabet: &ValidatedAlphabet,
    rect: &Rect,
    pointer_pos: egui::Pos2,
    painter: &egui::Painter,
) -> Option<SelectableKey> {
    let zoom = viewport.camera_zoom.get();
    let size = NODE_SIZE * zoom;
    let pointer_position_rect = Rect::from_center_size(pointer_pos, Vec2::new(10.0, 10.0));

    for (key, position) in graph.nodes_iter() {
        let screen_position: Pos2 = viewport.world_to_screen(*position).into();
        if !rect.contains(screen_position) {
            continue;
        }

        if pointer_position_rect.intersects(Rect::from_center_size(
            screen_position,
            Vec2::new(size, size),
        )) {
            return Some(SelectableKey::Node(key));
        }
    }

    if !rect.contains(pointer_pos) {
        return None;
    }
    for (key, note) in graph.notes_iter() {
        if note_rect(note, viewport, painter).contains(pointer_pos) {
            return Some(SelectableKey::Note(key));
        }
    }

    for (key, edge) in submachine.edges_iter() {
        let source = viewport.world_to_screen(
            graph
                .get_node_position(edge.source())
                .expect("Unreachable. Edge can't exist without source"),
        );

        let target = viewport.world_to_screen(
            graph
                .get_node_position(edge.target())
                .expect("Unreachable. Edge can't exist without target"),
        );

        let pts = build_edge_points(
            source.into(),
            target.into(),
            edge.source() == edge.target(),
            zoom,
        );

        let hit_distance = 6.0;

        if (!edge.accepts_all(alphabet)
            && edge_label_layout(painter, &pts, edge, zoom, egui::Color32::BLACK)
                .rect
                .contains(pointer_pos))
            || pts
                .windows(2)
                .any(|w| distance_to_segment(pointer_pos, w[0], w[1]) <= hit_distance)
        {
            return Some(SelectableKey::Edge(key));
        }
    }

    None
}

fn render_selection_rect(painter: &egui::Painter, start: egui::Pos2, end: egui::Pos2) {
    let rect = egui::Rect::from_two_pos(start, end);

    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(
            SELECTED_NODE_FILL.r(),
            SELECTED_NODE_FILL.g(),
            SELECTED_NODE_FILL.b(),
            35,
        ),
    );
}

fn select_items_in_rect(
    graph: &GraphLayout,
    viewport: &ViewportState,
    select_rect: Rect,
) -> Vec<SelectableKey> {
    let mut selected = Vec::new();

    selected.extend(graph.nodes_iter().filter_map(|(key, position)| {
        let screen_position: Pos2 = viewport.world_to_screen(*position).into();

        select_rect
            .contains(screen_position)
            .then_some(SelectableKey::Node(key))
    }));

    selected.extend(graph.notes_iter().filter_map(|(key, note)| {
        let screen_position: Pos2 = viewport.world_to_screen(note.position).into();

        select_rect
            .contains(screen_position)
            .then_some(SelectableKey::Note(key))
    }));

    selected
}

fn render_drag_snap_preview(
    painter: &egui::Painter,
    document: &SubmachineDocument,
    viewport: &ViewportState,
    interaction: &CanvasInteractionState,
) {
    let zoom = viewport.camera_zoom.get();

    for item in &interaction.drag {
        let rect = match item {
            SelectableKey::Node(key) => {
                let Some(position) = document.layout().get_node_position(*key) else {
                    continue;
                };
                let screen = viewport.world_to_screen(GraphLayout::snap_to_grid(position));
                egui::Rect::from_center_size(screen.into(), egui::Vec2::splat(NODE_SIZE * zoom))
            }
            SelectableKey::Note(key) => {
                let Some((_, note)) = document.layout().notes_iter().find(|(id, _)| id == key)
                else {
                    continue;
                };
                let mut snapped = note.clone();
                snapped.position = GraphLayout::snap_to_grid(note.position);
                note_rect(&snapped, viewport, painter)
            }
            SelectableKey::Edge(_) => continue,
        };

        painter.rect_filled(
            rect,
            4.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 50),
        );

        painter.rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120)),
            egui::StrokeKind::Inside,
        );
    }
}
