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
                ui.add(egui::TextEdit::singleline(&mut self.video_path).desired_width(f32::INFINITY));
            });
            ui.horizontal(|ui| {
                if ui.button("Subtitle").clicked() {
                    self.subtitle_path = select_file();
                }
                ui.add(egui::TextEdit::singleline(&mut self.subtitle_path).desired_width(f32::INFINITY));
            });

            ui.label(egui::RichText::new("Subtitles").heading());
            if !self.subtitle_path.is_empty() {
                // XXX: There are multiple file formats for subtitles including SRT and VTT.
                //      We should ensure nice handling of all formats here.
                //      If the user provides a bad file path, lets give them a red warning box
                //      telling them to double check the extension of their file.
                let items = match srtparse::from_file(&self.subtitle_path) {
                    Ok(subtitles) => subtitles,
                    Err(error) => {
                        let frame = egui::Frame::window(&ui.style())
                            .shadow(egui::Shadow::NONE)
                            .fill(egui::Color32::LIGHT_RED)
                            .stroke(egui::Stroke::new(2.0, egui::Color32::RED));
                        frame.show(ui, |ui| {
                            ui.label(format!(
                                "Unable to parse {} due to error: {}",
                                &self.subtitle_path, error
                            ));
                        });
                        Vec::new()
                    }
                };
                let subtitles = items
                    .iter()
                    .map(|sub| sub.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");
                ui.label(subtitles);
                // println!("{:?}", items);
            }
        });
    }
}

fn select_file() -> String {
    match rfd::FileDialog::new().pick_file() {
        Some(file) => file.display().to_string(),
        None => "".into(),
    }
}