use egui::{Response, RichText, Ui, Vec2};

use crate::ui::theme::{
    BUTTON_SIZE, BUTTON_STROKE_WIDTH, ICON_FONT_SIZE, SELECTED_NODE_FILL, SELECTED_NODE_STROKE,
    SUBSCRIPT_FONT_SIZE,
};

pub fn icon_button(ui: &mut Ui, icon: &str, size: Vec2) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let visuals = ui.style().interact(&response);
    ui.painter().rect(
        rect,
        visuals.corner_radius,
        visuals.weak_bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );
    let color = if ui.is_enabled() {
        ui.visuals().text_color()
    } else {
        ui.visuals().weak_text_color()
    };
    paint_icon(ui.painter(), rect.center(), 16., icon, color);
    response
}

pub fn centered_tool_grid(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    add_contents: impl FnOnce(&mut Ui),
) {
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - 168.) / 2.).max(0.));
        egui::Grid::new(id)
            .num_columns(2)
            .min_col_width(76.)
            .max_col_width(76.)
            .spacing([16., 10.])
            .show(ui, add_contents);
    });
}

pub fn tool_button(
    ui: &mut Ui,
    icon: &str,
    label: &str,
    shortcut: Option<&str>,
    is_selected: bool,
) -> Response {
    ui.allocate_ui_with_layout(
        egui::vec2(76., BUTTON_SIZE + SUBSCRIPT_FONT_SIZE + 8.),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            let mut button = egui::Button::new("").frame(true);
            if is_selected {
                button = button
                    .fill(SELECTED_NODE_FILL)
                    .stroke(egui::Stroke::new(BUTTON_STROKE_WIDTH, SELECTED_NODE_STROKE));
            }
            let response = ui.add_sized(Vec2::splat(BUTTON_SIZE), button);
            paint_icon(
                ui.painter(),
                response.rect.center(),
                ICON_FONT_SIZE,
                icon,
                if !ui.is_enabled() {
                    ui.visuals().weak_text_color()
                } else {
                    ui.visuals().text_color()
                },
            );

            if let Some(shortcut) = shortcut {
                let rect = response.rect;

                let galley = ui.painter().layout_no_wrap(
                    shortcut.to_owned(),
                    egui::FontId::proportional(10.0),
                    ui.visuals().weak_text_color(),
                );

                let padding = 3.0;
                let text_pos = egui::pos2(
                    rect.right() - galley.size().x - padding,
                    rect.top() + padding,
                );

                ui.painter()
                    .galley(text_pos, galley, ui.visuals().weak_text_color());
            }

            ui.label(RichText::new(label).size(SUBSCRIPT_FONT_SIZE));

            response
        },
    )
    .inner
}

pub fn paint_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    size: f32,
    icon: &str,
    color: egui::Color32,
) {
    let scale = size / 20.;
    let point = |x: f32, y: f32| center + egui::vec2(x * scale, y * scale);
    let stroke = egui::Stroke::new((size / 13.).max(1.2), color);
    let line = |points: &[(f32, f32)]| {
        painter.add(egui::Shape::line(
            points.iter().map(|&(x, y)| point(x, y)).collect(),
            stroke,
        ));
    };
    match icon {
        "↗" => {
            line(&[(-6., 6.), (6., -6.)]);
            line(&[(-2., -6.), (6., -6.), (6., 2.)]);
        }
        "⚙" => {
            painter.circle_stroke(center, 5. * scale, stroke);
            painter.circle_stroke(center, 2. * scale, stroke);
            for index in 0..8 {
                let angle = index as f32 * std::f32::consts::TAU / 8.;
                let direction = egui::vec2(angle.cos(), angle.sin());
                painter.line_segment(
                    [
                        center + direction * (5. * scale),
                        center + direction * (8. * scale),
                    ],
                    stroke,
                );
            }
        }
        "▣" => {
            line(&[
                (-7., -7.),
                (5., -7.),
                (7., -5.),
                (7., 7.),
                (-7., 7.),
                (-7., -7.),
            ]);
            line(&[(-3., -7.), (-3., -1.), (4., -1.), (4., -7.)]);
            line(&[(-4., 7.), (-4., 2.), (4., 2.), (4., 7.)]);
        }
        "↑" | "↓" => {
            line(&[(-7., 2.), (-7., 7.), (7., 7.), (7., 2.)]);
            if icon == "↑" {
                line(&[(0., 3.), (0., -7.)]);
                line(&[(-4., -3.), (0., -7.), (4., -3.)]);
            } else {
                line(&[(0., -7.), (0., 3.)]);
                line(&[(-4., -1.), (0., 3.), (4., -1.)]);
            }
        }
        "↶" | "↷" => {
            let sign = if icon == "↶" { 1. } else { -1. };
            let pts = [
                (5. * sign, 6.),
                (7. * sign, 1.),
                (5. * sign, -4.),
                (-sign, -5.),
                (-7. * sign, -1.),
            ];
            line(&pts);
            line(&[(-7. * sign, -6.), (-7. * sign, -1.), (-2. * sign, -1.)]);
        }
        "⌂" => {
            line(&[(-8., 0.), (0., -7.), (8., 0.)]);
            line(&[(-6., -1.), (-6., 7.), (6., 7.), (6., -1.)]);
            line(&[(-2., 7.), (-2., 2.), (2., 2.), (2., 7.)]);
        }
        "▶" | "▷" => {
            painter.add(egui::Shape::convex_polygon(
                vec![point(-5., -7.), point(6., 0.), point(-5., 7.)],
                color,
                egui::Stroke::NONE,
            ));
            if icon == "▷" {
                line(&[(8., -7.), (8., 7.)]);
            }
        }
        "Ⅱ" => {
            line(&[(-4., -7.), (-4., 7.)]);
            line(&[(4., -7.), (4., 7.)]);
        }
        "■" => {
            painter.rect_filled(
                egui::Rect::from_center_size(center, egui::Vec2::splat(12. * scale)),
                1.,
                color,
            );
        }
        "✓" => line(&[(-7., 0.), (-2., 5.), (7., -6.)]),
        "←" => {
            line(&[(7., 0.), (-7., 0.)]);
            line(&[(-1., -6.), (-7., 0.), (-1., 6.)]);
        }
        "◎" => {
            painter.circle_stroke(center, 7. * scale, stroke);
            painter.circle_stroke(center, 3. * scale, stroke);
        }
        "≪" | "‹" | "›" | "≫" => {
            let sign = if icon == "≪" || icon == "‹" {
                1.
            } else {
                -1.
            };
            if icon == "≪" || icon == "≫" {
                line(&[(1. * sign, -5.), (-4. * sign, 0.), (1. * sign, 5.)]);
                line(&[(7. * sign, -5.), (2. * sign, 0.), (7. * sign, 5.)]);
            } else {
                line(&[(3. * sign, -5.), (-2. * sign, 0.), (3. * sign, 5.)]);
            }
        }
        _ => {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(size),
                color,
            );
        }
    }
}
