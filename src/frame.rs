use eframe::egui;

pub fn frame<R>(
    title: impl Into<egui::WidgetText>,
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let frame = egui::Frame::new().inner_margin(egui::Margin::same(6));

    frame.show(ui, |ui| {
        ui.label(title);
        ui.separator();
        add_contents(ui)
    })
}
