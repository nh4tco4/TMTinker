use egui::{Pos2, Rect, Vec2};

use crate::{
    core::{ValidatedAlphabet, tm::Submachine}, editor::{
        layout::{GraphLayout, LayoutPos}, session::{CanvasInteractionState, RectSelection, SelectableKey, ViewportState, Zoom},
    }, project::SubmachineDocument, visual::{
        mappings::{NODE_SIZE, SELECTED_NODE_FILL}, render::{ render_edge::{build_edge_points, render_edges},
            render_grid::render_grid,
            render_node::render_nodes,
            render_note::render_notes,
        }, toolbars::{edge_control_window::edge_options_base, node_control_windows::node_options_base}, tools::{
            basic_node_tools::{left_tool, right_tool, start_tool, stop_tool, submachine_tool, write_tool}, dyn_tools_base::{BasicActionTools, EditTools, EditorTool}, edit_tools::{delete_tool, link_tool, note_tool},
        }
    },
};

pub fn handle_canvas(
    submachine_document: &mut SubmachineDocument,
    current_tool: &EditorTool,
    alphabet: ValidatedAlphabet,
    ctx: &egui::Context,
    ui: &mut egui::Ui
) {
    let (resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    let rect = resp.rect;

    // Handle zoom
    {
        let zoom_delta = ui.input(|i| i.zoom_delta());
        if zoom_delta != 1.0 {
            let old_zoom = submachine_document.viewport.camera_zoom.get();
            let new_zoom = Zoom::new(submachine_document.viewport.camera_zoom.get() * zoom_delta);

            if let Some(cursor_screen) = ctx.input(|i| i.pointer.hover_pos()) {
                let cursor_screen: LayoutPos = cursor_screen.into();
                let cursor_world: LayoutPos = (cursor_screen - submachine_document.viewport.camera_pan) * (1.0 / old_zoom);

                let new_pan: LayoutPos = cursor_screen - cursor_world * new_zoom.get();

                submachine_document.viewport.camera_zoom = new_zoom;
                submachine_document.viewport.camera_pan = new_pan;
            } else {
                submachine_document.viewport.camera_zoom = new_zoom;
            }
        }
    }

    let id = ui.id().with("canvas_interaction_state");

    let mut canvas_interaction_state = ctx.data_mut(|d| {
        d.get_temp::<CanvasInteractionState>(id)
            .unwrap_or_default()
    });

    if let Some(pointer_position) = ctx.input(|i| i.pointer.hover_pos()) {
        let hovered_key = compute_hovered_key(
            &submachine_document.layout,
            &submachine_document.viewport,
            &submachine_document.machine,
            &rect,
            pointer_position
        );

        // Handle canvas drag with LMB
        //  TODO turn it into function
        {
            if resp.drag_started_by(egui::PointerButton::Primary) {
                match hovered_key {
                    Some(SelectableKey::Node(key)) => {
                        let item = SelectableKey::Node(key);

                        if !canvas_interaction_state
                            .selection_state
                            .is_selected(&item)
                        {
                            if !ctx.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                                canvas_interaction_state.selection_state.clear();
                            }

                            canvas_interaction_state
                                .selection_state
                                .single_select(item);
                        }

                        canvas_interaction_state.drag =
                            canvas_interaction_state.selection_state.selected.clone();
                    }
                    Some(SelectableKey::Note(key)) => {
                        let item = SelectableKey::Note(key);

                        if !canvas_interaction_state
                            .selection_state
                            .is_selected(&item)
                        {
                            if !ctx.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                                canvas_interaction_state.selection_state.clear();
                            }

                            canvas_interaction_state
                                .selection_state
                                .single_select(item);
                        }

                        canvas_interaction_state.drag = vec![item];
                    }
                    _ => {
                        canvas_interaction_state.drag.drain(..).for_each(|i| 
                            match i {
                                SelectableKey::Node(key) => submachine_document.layout.update_node_position(key),
                                SelectableKey::Note(key) => submachine_document.layout.update_note_position(key),
                                _ => (),
                            }
                        );
                    }
                }
            }
        }

        // Handle rectangle selection
        // TODO turn it into function
        {
            if resp.drag_started_by(egui::PointerButton::Secondary) {
                canvas_interaction_state.rect_select = Some(RectSelection { start: pointer_position, current: pointer_position });
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
                    let select_rect =
                        egui::Rect::from_two_pos(rectangle.start, rectangle.current);

                    let selected = select_items_in_rect(
                        &submachine_document.layout,
                        &submachine_document.viewport,
                        select_rect,
                    );

                    canvas_interaction_state.selection_state.selected = selected;
                }
            }
        }

        // Handle click
        // TODO turn it into function
        {
            if resp.clicked() /* && !resp.double_clicked() */ {
                let modifiers = ctx.input(|i| i.modifiers);

        match hovered_key {
            // Some(SelectableKey::Edge(key)) => {
            //     canvas_interaction_state
            //         .selection_state
            //         .single_select(SelectableKey::Edge(key));
            // }

            Some(id) => {
                if modifiers.shift || modifiers.command {
                    canvas_interaction_state
                        .selection_state
                        .toggle_selected(id);
                } else {
                    canvas_interaction_state
                        .selection_state
                        .single_select(id);
                }
            }

            None => {
                if !modifiers.ctrl
                    && !modifiers.shift
                    && !modifiers.command
                {
                    canvas_interaction_state
                        .selection_state
                        .clear();
                }
            }
        }

                let shift_pressed = ctx.input(|i| i.modifiers.shift);

                handle_click(
                    submachine_document,
                    current_tool,
                    pointer_position,
                    &mut canvas_interaction_state,
                    shift_pressed,
                    &alphabet,
                    );
            }
        }

        if let Some(node_key) = canvas_interaction_state.selection_state.single_node() && *current_tool != EditorTool::Edit(EditTools::Link) {
            if let Some(node_position) = submachine_document.layout.get_node_position(node_key) {
                let window_pos = submachine_document.viewport.world_to_screen(node_position)
                    * submachine_document.viewport.camera_zoom.get()
                    + submachine_document.viewport.camera_pan;

                node_options_base(
                    &submachine_document.viewport,
                    &mut submachine_document.machine,
                    &alphabet,
                    ctx,
                    node_key,
                    window_pos.into(),
                    );
            }
        }

        if let Some(edge_key) = canvas_interaction_state.selection_state.single_edge() {
            let window_pos = pointer_position;

            edge_options_base(
                &submachine_document.viewport,
                &mut submachine_document.machine,
                &alphabet,
                ctx,
                edge_key,
                window_pos.into(),
                );
        }
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
            if modifiers.shift {
                submachine_document.viewport.camera_pan.x += scroll.x * submachine_document.viewport.camera_zoom.get() * 3.0;
            } else {
                submachine_document.viewport.camera_pan.y += scroll.y * submachine_document.viewport.camera_zoom.get() * 3.0;
            }
        }
    }

    // Dragging
    // TODO turn it into function
        {
            if resp.dragged_by(egui::PointerButton::Primary) {
                if !canvas_interaction_state.drag.is_empty() {
                    let zoom = submachine_document.viewport.camera_zoom.get();
                    let delta = resp.drag_delta();

                    let delta = LayoutPos {
                        x: delta.x / zoom,
                        y: delta.y / zoom,
                    };

                    for item in &canvas_interaction_state.drag {
                        match item {
                            SelectableKey::Node(key) => {
                                submachine_document
                                    .layout
                                    .update_node_position_unsnapped(*key, delta);
                            }

                            SelectableKey::Note(_key) => {}

                            SelectableKey::Edge(_) => {}
                        }
                    }
                } else {
                    submachine_document.viewport.camera_pan.x += resp.drag_delta().x;
                    submachine_document.viewport.camera_pan.y += resp.drag_delta().y;
                }
            }

            if resp.drag_stopped_by(egui::PointerButton::Primary) {
                for item in canvas_interaction_state.drag.drain(..) {
                    match item {
                        SelectableKey::Node(key) => {
                            submachine_document.layout.update_node_position(key);
                        }

                        SelectableKey::Note(key) => {
                            submachine_document.layout.update_note_position(key);
                        }

                        SelectableKey::Edge(_) => {}
                    }
                }
            }
        }


    // TODO make node edit window
    // if resp.double_clicked() {
    //     if let Some(SelectableKey::Note(note_id)) = app.current_editor().selected.hovered {
    //         app.editing_note = Some(note_id);
    //     }
    // }

    // Render
    {
        render_grid(&submachine_document.viewport, rect, &painter);

        if let Some(rectangle) = canvas_interaction_state.rect_select.as_ref() {
            render_selection_rect(&painter, rectangle.start, rectangle.current);
        }

        render_edges(
            &submachine_document.layout,
            &submachine_document.machine,
            &submachine_document.viewport,
            &canvas_interaction_state.selection_state,
            &painter,
        );

        render_drag_snap_preview(
            &painter,
            submachine_document,
            &canvas_interaction_state,
        );

        render_nodes(&submachine_document.layout, &submachine_document.machine, &submachine_document.viewport, &canvas_interaction_state.selection_state, &painter);

        render_notes(&submachine_document.layout, &submachine_document.viewport, &painter);
    }

    ctx.data_mut(|d| {
        d.insert_temp(id, canvas_interaction_state);
    });

    // if let Some(note_id) = app.editing_note {
    //     let mut open = true;
    //     let mut should_close = false;
    //
    //     let window_response = egui::Window::new("Edit Note")
    //         .id(egui::Id::new("edit_note_window"))
    //         .open(&mut open)
    //         .show(ctx, |ui| {
    //             let editor = app.current_editor_mut();
    //             if let Some(note) = editor.notes.notes.get_mut(&note_id) {
    //                 ui.text_edit_multiline(&mut note.content);
    //             }
    //         });
    //
    //     if let Some(response) = window_response {
    //         if response.response.lost_focus() && ctx.input(|i| i.pointer.any_click()) {
    //             should_close = true;
    //         }
    //     }
    //
    //     if !open || should_close {
    //         app.editing_note = None;
    //     }
    // }
}

fn handle_click(
    submachine_document: &mut SubmachineDocument,
    current_tool: &EditorTool,
    pointer_position: egui::Pos2,
    canvas_interaction_state: &mut CanvasInteractionState,
    _shift_pressed: bool,
    alphabet: &ValidatedAlphabet
) {
    let world = submachine_document
        .viewport
        .screen_to_world(pointer_position.into());

    let snapped = GraphLayout::snap_to_grid(world).into();

    // let to_select = !shift_pressed;
    match current_tool {
        EditorTool::Place(BasicActionTools::Left) => {
            left_tool(submachine_document, snapped);
        },
        EditorTool::Place(BasicActionTools::Right) => {
            right_tool(submachine_document, snapped);
        },
        EditorTool::Place(BasicActionTools::Write) => {
            write_tool(submachine_document, snapped); 
        },
        EditorTool::Place(BasicActionTools::Start) => {
            start_tool(submachine_document, snapped);
        },
        EditorTool::Place(BasicActionTools::Stop) => {
            stop_tool(submachine_document, snapped);
        },
        EditorTool::Edit(EditTools::Comment) => {
            note_tool(submachine_document, snapped.into());
        },
        EditorTool::Edit(EditTools::Delete) => {
            if let Some(key) = canvas_interaction_state.selection_state.selected.get(0) {
                delete_tool(submachine_document, *key);
            } else {}
        },
        EditorTool::Edit(EditTools::Link) => {
            if canvas_interaction_state.selection_state.selected.len() == 1 {
                link_tool(
                    submachine_document,
                    &mut canvas_interaction_state.link_source,
                    alphabet,
                    canvas_interaction_state.selection_state.selected.get(0).copied()
                );
            }
        },
        EditorTool::Edit(EditTools::Move) => {}
        EditorTool::Submachine {
            name,
            key,
            power: _,
        } => {
            submachine_tool(submachine_document, snapped, key, name.to_owned());
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

fn compute_hovered_key(graph: &GraphLayout, viewport: &ViewportState, submachine: &Submachine, rect: &Rect, pointer_pos: egui::Pos2) -> Option<SelectableKey> {
    let zoom = viewport.camera_zoom.get();
    let size = NODE_SIZE * zoom;
    let pointer_position_rect = Rect::from_center_size(pointer_pos, Vec2::new(10.0, 10.0));

    for (key, position) in graph.nodes_iter() {
        let screen_position: Pos2 = viewport.world_to_screen(*position).into();
        if !rect.contains(screen_position) {
            continue;
        }

        if pointer_position_rect.intersects(Rect::from_center_size(screen_position, Vec2::new(size, size))) {
            return Some(SelectableKey::Node(key));
        }
    }

    // TODO figure out how to detect if nodes are hovered
    // for (key, note) in graph.notes_iter() {
    //     let screen_position: Pos2 = viewport.world_to_screen(note.position).into();
    //     if !rect.contains(screen_position) {
    //         continue;
    //     }
    //
    //     if pointer_position_rect.contains(screen_position) {
    //         return Some(SelectableKey::Node(key));
    //     }
    // }

    for (key, edge) in submachine.edges_iter() {
        let source = viewport.world_to_screen(
            graph
                .get_node_position(edge.source)
                .expect("Unreachable. Edge can't exist without source"),
        );

        let target = viewport.world_to_screen(
            graph
                .get_node_position(edge.target)
                .expect("Unreachable. Edge can't exist without target"),
        );

        let pts = build_edge_points(
            source.into(),
            target.into(),
            edge.source == edge.target,
            zoom,
        );

        let hit_distance = 6.0;

        if pts
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
        // God forgive me for all of my sins
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

    selected.extend(
        graph.nodes_iter().filter_map(|(key, position)| {
            let screen_position: Pos2 =
                viewport.world_to_screen(*position).into();

            select_rect
                .contains(screen_position)
                .then_some(SelectableKey::Node(key))
        }),
    );

    selected.extend(
        graph.notes_iter().filter_map(|(key, note)| {
            let screen_position: Pos2 =
                viewport.world_to_screen(note.position).into();

            select_rect
                .contains(screen_position)
                .then_some(SelectableKey::Note(key))
        }),
    );

    selected
}

fn render_drag_snap_preview(
    painter: &egui::Painter,
    document: &SubmachineDocument,
    interaction: &CanvasInteractionState,
) {
    let zoom = document.viewport.camera_zoom.get();

    for item in &interaction.drag {
        let SelectableKey::Node(key) = item else {
            continue;
        };

        let Some(position) = document.layout.get_node_position(*key) else {
            continue;
        };

        let snapped = GraphLayout::snap_to_grid(position);

        let screen: egui::Pos2 =
            document.viewport.world_to_screen(snapped).into();

        let size = NODE_SIZE * zoom;

        let rect = egui::Rect::from_center_size(
            screen,
            egui::vec2(size, size),
        );

        painter.rect_filled(
            rect,
            4.0,
            egui::Color32::from_rgba_unmultiplied(
                    0,
                    0,
                    0,
                50,
            ),
        );

        painter.rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(
                    0,
                    0,
                    0,
                    120,
                ),
            ),
            egui::StrokeKind::Inside,
        );
    }
}
