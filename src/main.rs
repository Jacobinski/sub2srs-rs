use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 240.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "sub2srs",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default)]
struct MyApp {
    video_path: String,
    subtitle_path: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(egui::RichText::new("Files").heading());
            ui.horizontal(|ui| {
                if ui.button("Video").clicked() {
                    self.video_path = select_file();
                }
                ui.text_edit_singleline(&mut self.video_path);
            });
            ui.horizontal(|ui| {
                if ui.button("Subtitle").clicked() {
                    self.subtitle_path = select_file();
                }
                ui.text_edit_singleline(&mut self.subtitle_path);
            });
        });
    }
}

fn select_file() -> String {
    match rfd::FileDialog::new().pick_file() {
        Some(file) => file.display().to_string(),
        None => "".into(),
    }
}
