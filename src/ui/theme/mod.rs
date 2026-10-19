mod icons;
mod style;

pub use icons::*;
pub use style::*;

pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 218))
        .stroke(egui::Stroke::new(1., egui::Color32::from_gray(214)))
        .corner_radius(18.)
        .inner_margin(24.)
}

pub fn panel_frame() -> egui::Frame {
    card_frame().inner_margin(12.).outer_margin(8.)
}

pub fn sidebar(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    panel_frame().show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        add_contents(ui);
    });
}
