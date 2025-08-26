use eframe::egui;

pub fn frame<R>(
    title: impl Into<egui::WidgetText>,
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin { left: 6, right: 6, top: 6, bottom: 6 })
        .corner_radius(5.0);

    frame.show(ui, |ui| {
        ui.label(title);
        ui.separator();
        add_contents(ui)
    })
}