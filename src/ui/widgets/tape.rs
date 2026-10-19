use crate::core::tape::{BLANK, Tape, TapeIndex};
use egui::{Color32, FontId, RichText};

pub struct TapeView<'a> {
    tape: &'a Tape,
    center: TapeIndex,
    radius: i32,
    selected: Option<TapeIndex>,
}
impl<'a> TapeView<'a> {
    pub fn new(tape: &'a Tape) -> Self {
        Self {
            tape,
            center: tape.head_position(),
            radius: 8,
            selected: None,
        }
    }
    pub fn centered(mut self, center: TapeIndex, radius: i32, selected: Option<TapeIndex>) -> Self {
        self.center = center;
        self.radius = radius;
        self.selected = selected;
        self
    }
    pub fn first_index(&self) -> i64 {
        i64::from(self.center) - i64::from(self.radius)
    }
}
impl egui::Widget for TapeView<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let width = ((self.radius * 2 + 1) as f32 * 46.).min(ui.available_width());
        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 76.), egui::Sense::click());
        let painter = ui.painter_at(rect);
        for offset in 0..=(self.radius * 2) {
            let index = self.first_index() + i64::from(offset);
            if index < i64::from(i32::MIN) || index > i64::from(i32::MAX) {
                continue;
            }
            let index = index as i32;
            let cell = egui::Rect::from_min_size(
                rect.min + egui::vec2(offset as f32 * 46. + 2., 8.),
                egui::vec2(42., 42.),
            );
            let head = index == self.tape.head_position();
            let selected = self.selected == Some(index);
            painter.rect_filled(
                cell,
                4.,
                if head {
                    crate::ui::theme::SELECTED_NODE_FILL
                } else {
                    Color32::from_rgb(245, 247, 249)
                },
            );
            painter.rect_stroke(
                cell,
                4.,
                egui::Stroke::new(
                    if head || selected { 2. } else { 1. },
                    if head || selected {
                        crate::ui::theme::SELECTED_NODE_STROKE
                    } else {
                        crate::ui::theme::PANEL_STROKE
                    },
                ),
                egui::StrokeKind::Inside,
            );
            let symbol = self.tape.cell(index);
            painter.text(
                cell.center(),
                egui::Align2::CENTER_CENTER,
                if symbol == BLANK {
                    "λ".into()
                } else {
                    symbol.to_string()
                },
                FontId::proportional(22.),
                crate::ui::theme::TEXT_COLOR,
            );
            painter.text(
                egui::pos2(cell.center().x, cell.bottom() + 7.),
                egui::Align2::CENTER_TOP,
                index.to_string(),
                FontId::proportional(11.),
                Color32::from_gray(115),
            );
            if head {
                painter.text(
                    egui::pos2(cell.center().x, cell.top() - 1.),
                    egui::Align2::CENTER_BOTTOM,
                    "▾",
                    FontId::proportional(13.),
                    crate::ui::theme::SELECTED_NODE_STROKE,
                );
            }
        }
        response
    }
}

#[derive(Clone)]
pub struct TapeDraft {
    pub text: String,
    pub origin: i32,
}
impl TapeDraft {
    pub fn from_tape(tape: &Tape) -> Self {
        let (text, origin) = tape.to_text();
        Self { text, origin }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut self.text)
                .font(egui::TextStyle::Body)
                .desired_width(f32::INFINITY),
        );
    }
    pub fn parse(
        &self,
        alphabet: &crate::core::alphabet::ValidatedAlphabet,
    ) -> Result<Tape, String> {
        Tape::from_text(&self.text, self.origin, alphabet)
    }
}
pub fn notation_hint(ui: &mut egui::Ui) {
    ui.label(
        RichText::new("Tape: 01(1)0 · ( ) marks the head · λ is blank")
            .small()
            .weak(),
    );
}
