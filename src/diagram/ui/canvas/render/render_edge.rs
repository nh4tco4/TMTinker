use std::sync::Arc;

use egui::{Painter, Pos2};

use crate::{
    core::alphabet::ValidatedAlphabet,
    diagram::editor::session::{SelectableKey, SelectionState, ViewportState},
    diagram::model::layout::GraphLayout,
    diagram::model::{
        submachine::Submachine,
        transition::{Edge, EdgeKey},
    },
    ui::theme::{
        ACTIVE_NODE_STROKE, BUTTON_SIZE, ICON_FONT_SIZE, NODE_SPACE, REGULAR_NODE_STROKE,
        SELECTED_NODE_STROKE,
    },
};

enum EdgeMeta {
    NextTransition,
    Idle,
    ConnectedToSelection,
    Selected,
    Hovered,
}

pub fn render_edges(
    graph: &GraphLayout,
    machine: &Submachine,
    alphabet: &ValidatedAlphabet,
    viewport: &ViewportState,
    selection_state: &SelectionState,
    next_transition: Option<EdgeKey>,
    ui: &egui::Ui,
) {
    let painter = ui.painter().clone();
    let hovered_key = match selection_state.hovered {
        Some(SelectableKey::Edge(edge_key)) => Some(edge_key),
        _ => None,
    };

    let selected_nodes = selection_state.selected_node_keys();

    for (key, edge) in machine.edges_iter() {
        let source_position = viewport.world_to_screen(
            graph
                .get_node_position(edge.source())
                .expect("Unreachable. Edge can't exist without source."),
        );

        let target_position = viewport.world_to_screen(
            graph
                .get_node_position(edge.target())
                .expect("Unreachable. Edge can't exist without target."),
        );

        let is_loop = edge.source() == edge.target();

        let points = build_edge_points(
            source_position.into(),
            target_position.into(),
            is_loop,
            viewport.camera_zoom.get(),
        );

        let is_hovered = Some(key) == hovered_key;

        let is_selected = selection_state.is_selected(&SelectableKey::Edge(key));

        let connects_selected_nodes =
            selected_nodes.contains(&edge.source()) && selected_nodes.contains(&edge.target());

        let edge_meta = if next_transition == Some(key) {
            EdgeMeta::NextTransition
        } else if is_selected {
            EdgeMeta::Selected
        } else if is_hovered {
            EdgeMeta::Hovered
        } else if connects_selected_nodes {
            EdgeMeta::ConnectedToSelection
        } else {
            EdgeMeta::Idle
        };

        let (stroke, color) = edge_style(edge_meta);

        draw_path(&painter, &points, stroke);

        painter.add(arrow_tip(&points, color, viewport.camera_zoom.get()));

        let dot_r = 5.0 * viewport.camera_zoom.get();

        painter.circle_filled(*points.first().unwrap(), dot_r, color);

        painter.circle_filled(*points.last().unwrap(), dot_r, color);

        if edge.accepts_all(alphabet) {
            continue;
        }

        let label = edge_label_layout(
            &painter,
            &points,
            edge,
            viewport.camera_zoom.get(),
            if next_transition == Some(key) {
                ACTIVE_NODE_STROKE
            } else {
                egui::Color32::BLACK
            },
        );
        // A small background keeps vertical segments from crossing the text.
        painter.rect_filled(label.rect.expand(2.0), 2.0, ui.visuals().panel_fill);
        painter.galley(label.rect.min, label.galley, color);
        ui.interact(
            label.rect,
            ui.id().with(("edge_label", key)),
            egui::Sense::hover(),
        )
        .on_hover_ui(|ui| {
            ui.set_max_width(360.0);
            if edge.inverse() {
                ui.label(format!(
                    "Excluded: {}",
                    display_symbols(edge.symbols().iter())
                ));
            }
            ui.label(format!(
                "Allowed: {}",
                display_symbols(
                    alphabet
                        .iter()
                        .copied()
                        .filter(|&c| edge.accepts(c, alphabet))
                )
            ));
        });
    }
}

fn edge_style(meta: EdgeMeta) -> (egui::Stroke, egui::Color32) {
    match meta {
        EdgeMeta::NextTransition => (
            egui::Stroke::new(3.5, ACTIVE_NODE_STROKE),
            ACTIVE_NODE_STROKE,
        ),
        EdgeMeta::Idle => (
            egui::Stroke::new(2.0, REGULAR_NODE_STROKE),
            REGULAR_NODE_STROKE,
        ),

        EdgeMeta::ConnectedToSelection => (
            egui::Stroke::new(2.5, SELECTED_NODE_STROKE),
            SELECTED_NODE_STROKE,
        ),

        EdgeMeta::Hovered => (
            egui::Stroke::new(2.5, SELECTED_NODE_STROKE),
            SELECTED_NODE_STROKE,
        ),

        EdgeMeta::Selected => (
            egui::Stroke::new(3.0, SELECTED_NODE_STROKE),
            SELECTED_NODE_STROKE,
        ),
    }
}

pub fn build_edge_points(src: Pos2, tgt: Pos2, is_loop: bool, zoom: f32) -> Vec<Pos2> {
    let node_size = BUTTON_SIZE * zoom;

    let src_mid_y = src.y;
    let tgt_mid_y = tgt.y;

    let start = Pos2::new(src.x + node_size / 2.0, src_mid_y);
    let end = Pos2::new(tgt.x - node_size / 2.0, tgt_mid_y);

    let mut points = vec![start];

    if is_loop {
        let ox = node_size;
        let oy = node_size * 2.0;

        points.extend([
            Pos2::new(start.x + ox, start.y),
            Pos2::new(start.x + ox, start.y + oy),
            Pos2::new(start.x - ox * 2.0, start.y + oy),
            Pos2::new(start.x - ox * 2.0, start.y),
            Pos2::new(start.x - ox, start.y),
        ]);
    } else if src.x + 100.0 * zoom < tgt.x {
        let mid_x = f32::midpoint(start.x, end.x);

        points.extend([
            Pos2::new(mid_x, src_mid_y),
            Pos2::new(mid_x, tgt_mid_y),
            end,
        ]);
    } else {
        let offset = node_size;
        let mid_y = f32::midpoint(src_mid_y, tgt_mid_y);

        points.extend([
            Pos2::new(src.x + offset, src_mid_y),
            Pos2::new(src.x + offset, mid_y),
            Pos2::new(tgt.x - offset, mid_y),
            Pos2::new(tgt.x - offset, tgt_mid_y),
            end,
        ]);
    }

    points
}

fn draw_path(painter: &Painter, points: &[Pos2], stroke: egui::Stroke) {
    for window in points.windows(2) {
        painter.line_segment([window[0], window[1]], stroke);
    }
}

fn arrow_tip(points: &[Pos2], color: egui::Color32, zoom: f32) -> egui::Shape {
    let tip = *points.last().expect("arrow_tip called with empty points");

    let prev = points[points.len() - 2];

    let arrow_width = 6.0 * zoom;
    let arrow_len = 10.0 * zoom;

    let dir = (tip - prev).normalized();
    let perp = egui::Vec2::new(-dir.y, dir.x) * arrow_width;
    let base = tip - dir * (arrow_len + 5.0);

    egui::Shape::convex_polygon(
        vec![tip, base + perp, base - perp],
        color,
        egui::Stroke::NONE,
    )
}

pub(crate) struct EdgeLabel {
    pub rect: egui::Rect,
    galley: Arc<egui::Galley>,
}

/// Use the same layout for painting, tooltips, and selecting an edge by its label.
pub(crate) fn edge_label_layout(
    painter: &Painter,
    points: &[Pos2],
    edge: &Edge,
    zoom: f32,
    color: egui::Color32,
) -> EdgeLabel {
    let font = egui::FontId::proportional(ICON_FONT_SIZE * zoom);
    let mut symbols: Vec<_> = edge.symbols().iter().collect();
    symbols.sort_unstable();
    let prefix = if edge.inverse() { "≠ " } else { "" };
    let text_for = |visible: usize| {
        let mut text = prefix.to_owned();
        if symbols.is_empty() {
            text.push('∅');
        }
        for &symbol in symbols.iter().take(visible) {
            if symbol == ' ' {
                text.push_str(NODE_SPACE);
            } else {
                text.push(symbol);
            }
        }
        if visible < symbols.len() {
            text.push_str(&format!("… +{}", symbols.len() - visible));
        }
        text
    };
    let mut visible = symbols.len();
    let galley = loop {
        let galley = painter.layout_no_wrap(text_for(visible), font.clone(), color);
        if galley.size().x <= 160.0 * zoom || visible == 0 {
            break galley;
        }
        visible -= 1;
    };
    let center = path_midpoint(points) + egui::vec2(0.0, -20.0 * zoom);
    EdgeLabel {
        rect: egui::Rect::from_center_size(center, galley.size()),
        galley,
    }
}

fn path_midpoint(points: &[Pos2]) -> Pos2 {
    let mut remaining = points
        .iter()
        .zip(points.iter().skip(1))
        .map(|(from, to)| from.distance(*to))
        .sum::<f32>()
        / 2.0;
    for (from, to) in points.iter().zip(points.iter().skip(1)) {
        let length = from.distance(*to);
        if length > 0.0 && remaining <= length {
            return *from + (*to - *from) * (remaining / length);
        }
        remaining -= length;
    }
    points.first().copied().unwrap_or(Pos2::ZERO)
}

fn display_symbols(symbols: impl IntoIterator<Item = char>) -> String {
    let mut symbols: Vec<_> = symbols.into_iter().collect();
    symbols.sort_unstable();
    if symbols.is_empty() {
        return "∅".to_owned();
    }
    symbols
        .into_iter()
        .map(|c| {
            if c == ' ' {
                NODE_SPACE.to_owned()
            } else {
                c.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagram::model::{
        node::NodeAction, submachine::SubmachineName, transition::TransitionSymbols,
    };

    #[test]
    fn labels_follow_the_middle_of_routed_edges_and_loops() {
        let points = [
            egui::pos2(0., 0.),
            egui::pos2(20., 0.),
            egui::pos2(20., 100.),
            egui::pos2(40., 100.),
        ];
        assert_eq!(path_midpoint(&points), egui::pos2(20., 50.));
        let points = build_edge_points(egui::pos2(0., 0.), egui::pos2(0., 0.), true, 1.0);
        let middle = path_midpoint(&points);
        assert!(middle.y > 0.0);
        assert!(middle.x.abs() < 0.01);
    }

    #[test]
    fn long_inverse_labels_fit_the_width_at_each_zoom() {
        let alphabet = ValidatedAlphabet::new(
            " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars(),
        )
        .expect("valid test fixture");
        let mut machine =
            Submachine::new(SubmachineName::parse("Labels".into()).expect("valid test fixture"));
        let node = machine.add_node(NodeAction::Start);
        let key = machine.add_edge(
            TransitionSymbols::new(alphabet.iter().copied(), &alphabet)
                .expect("valid test fixture"),
            node,
            node,
        );
        let edge = machine.get_edge_mut(key).expect("valid test fixture");
        edge.set_inverse(true);
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                for zoom in [0.5, 1.0, 2.0] {
                    let label = edge_label_layout(
                        ui.painter(),
                        &[egui::pos2(0., 0.), egui::pos2(400., 0.)],
                        edge,
                        zoom,
                        egui::Color32::BLACK,
                    );
                    assert!(label.rect.width() <= 160.0 * zoom);
                    assert!(label.galley.job.text.starts_with("≠ "));
                    assert!(label.galley.job.text.contains("… +"));
                }
                edge.set_symbols(
                    TransitionSymbols::new([' '], &alphabet).expect("valid test fixture"),
                );
                let label =
                    edge_label_layout(ui.painter(), &[Pos2::ZERO], edge, 1.0, egui::Color32::BLACK);
                assert_eq!(label.galley.job.text, format!("≠ {NODE_SPACE}"));
            });
        });
    }

    #[test]
    fn hovering_a_shortened_label_shows_the_complete_set() {
        use crate::{diagram::model::layout::LayoutPos, ui::test_renderer::text_rects};
        let alphabet = ValidatedAlphabet::new(
            " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars(),
        )
        .expect("valid alphabet");
        let mut machine =
            Submachine::new(SubmachineName::parse("Hover".into()).expect("valid name"));
        let source = machine.add_node(NodeAction::Start);
        let target = machine.add_node(NodeAction::Stop);
        let key = machine.add_edge(
            TransitionSymbols::new(alphabet.iter().copied().filter(|c| *c != 'z'), &alphabet)
                .expect("valid symbols"),
            source,
            target,
        );
        let mut graph = GraphLayout::new();
        graph.set_node_position(source, LayoutPos::new(100., 150.));
        graph.set_node_position(target, LayoutPos::new(600., 150.));
        for inverse in [false, true] {
            machine
                .get_edge_mut(key)
                .expect("existing edge")
                .set_inverse(inverse);
            let ctx = egui::Context::default();
            let render = |time, events| {
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            Pos2::ZERO,
                            egui::vec2(800., 400.),
                        )),
                        time: Some(time),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            let _canvas = ui.allocate_painter(
                                ui.available_size(),
                                egui::Sense::click_and_drag(),
                            );
                            render_edges(
                                &graph,
                                &machine,
                                &alphabet,
                                &ViewportState::new(),
                                &SelectionState::default(),
                                None,
                                ui,
                            );
                        });
                    },
                )
            };
            let output = render(0.0, vec![]);
            let rect = text_rects(&output)
                .into_iter()
                .find(|(text, _)| text.contains("… +"))
                .expect("shortened label")
                .1;
            let _moved = render(0.1, vec![egui::Event::PointerMoved(rect.center())]);
            let _settled = render(0.2, vec![]);
            let _tooltip_layout = render(1.0, vec![]);
            let output = render(1.1, vec![]);
            let texts = text_rects(&output);
            let expected = format!(
                "{}: {}",
                if inverse { "Excluded" } else { "Allowed" },
                display_symbols(alphabet.iter().copied().filter(|c| *c != 'z'))
            );
            assert!(
                texts.iter().any(|(text, _)| text == &expected),
                "tooltip contains the complete set: {texts:?}"
            );
            if inverse {
                assert!(
                    texts.iter().any(|(text, _)| text == "Allowed: z"),
                    "tooltip includes the effective allowed set"
                );
            }
        }
    }

    #[test]
    fn transitions_accepting_the_entire_alphabet_have_no_label() {
        use crate::{diagram::model::layout::LayoutPos, ui::test_renderer::text_rects};
        let alphabet = ValidatedAlphabet::new([' ', '0', '1']).expect("valid alphabet");
        let mut machine =
            Submachine::new(SubmachineName::parse("Labels".into()).expect("valid name"));
        let source = machine.add_node(NodeAction::Start);
        let target = machine.add_node(NodeAction::Stop);
        let key = machine.add_edge(
            TransitionSymbols::new([' '], &alphabet).expect("valid symbols"),
            source,
            target,
        );
        let mut graph = GraphLayout::new();
        graph.set_node_position(source, LayoutPos::new(100., 150.));
        graph.set_node_position(target, LayoutPos::new(600., 150.));
        for (symbols, inverse, hidden) in [
            (vec![' ', '0', '1'], false, true),
            (vec![], true, true),
            (vec![' ', '0', '1'], true, false),
            (vec![' '], false, false),
        ] {
            let edge = machine.get_edge_mut(key).expect("existing edge");
            edge.set_symbols(
                TransitionSymbols::new_exclusions(symbols, &alphabet).expect("valid symbols"),
            );
            edge.set_inverse(inverse);
            let ctx = egui::Context::default();
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render_edges(
                        &graph,
                        &machine,
                        &alphabet,
                        &ViewportState::new(),
                        &SelectionState::default(),
                        None,
                        ui,
                    );
                });
            });
            assert_eq!(
                text_rects(&output).is_empty(),
                hidden,
                "hide precisely the conditions that accept every symbol"
            );
        }
    }
}
