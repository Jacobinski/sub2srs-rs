pub mod audio;
pub mod ffmpeg;
pub mod frame;
pub mod screenshot;

use eframe::egui;
use std::fs;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};

pub fn run() -> eframe::Result {
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

#[derive(Debug, Default, Clone)]
struct SubtitleClip {
    index: usize,
    start_time: std::time::Duration,
    end_time: std::time::Duration,
}

struct MyApp {
    tx: Sender<u32>,
    rx: Receiver<u32>,
    video_path: String,
    subtitle_path: String,
    clips: Vec<SubtitleClip>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            tx,
            rx,
            video_path: String::new(),
            subtitle_path: String::new(),
            clips: Vec::new(),
        }
    }
}

fn convert_subs_to_clips(subs: &[srtparse::Item]) -> Vec<SubtitleClip> {
    subs.iter()
        .map(|sub| SubtitleClip {
            index: sub.pos,
            start_time: sub.start_time.into_duration(),
            end_time: sub.end_time.into_duration(),
        })
        .collect()
}

fn process_clip(input_path: &String, output_dir: &String, clip: &SubtitleClip) {
    let start_time = clip.start_time.as_secs_f64();
    let end_time = clip.end_time.as_secs_f64();
    let mid_time = (clip.start_time + (clip.end_time - clip.start_time) / 2).as_secs_f64();
    let screenshot_path = Path::new(output_dir).join(format!("screenshot_{}.png", clip.index));
    let audio_path = Path::new(output_dir).join(format!("audio_clip_{}.mp3", clip.index));

    // XXX: We should check the results of these two calls.
    // TODO: Change these modules to use a PathBuf so we don't need to
    //       use this to_str().unwrap().to_string() garbage.
    screenshot::take_screenshot(
        mid_time,
        input_path.clone(),
        screenshot_path.to_str().unwrap().to_string(),
    );
    audio::record_audio_clip(
        start_time,
        end_time,
        input_path.clone(),
        audio_path.to_str().unwrap().to_string(),
    );
}

impl MyApp {
    fn generate_clips(&self) {
        let output_dir = "/tmp/sub2srs_test";
        if !Path::new(output_dir).exists() {
            fs::create_dir_all(output_dir).expect("Failed to create output directory");
        }

        for clip in &self.clips {
            process_clip(&self.video_path, &output_dir.to_string(), &clip);
        }
    }

    fn render_app(&mut self, ctx: &egui::Context) {
        // Add the ability to close on "esc" to improve the dev experience.
        // TODO: Remove this after launch, as at least add a pop-up warning.
        close_on_esc(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_ui(ui);
        });
    }

    fn render_ui(&mut self, ui: &mut egui::Ui) {
        frame::frame("Files", ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Video").clicked() {
                    self.video_path = select_file();
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.video_path).desired_width(f32::INFINITY),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("Subtitle").clicked() {
                    self.subtitle_path = select_file();
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.subtitle_path)
                        .desired_width(f32::INFINITY),
                );
            });
        });

        frame::frame("Subtitles", ui, |ui| {
            if !self.subtitle_path.is_empty() {
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
                self.clips = convert_subs_to_clips(&items);
            }

            let generate_button = egui::Button::new("Generate Clips");
            if ui
                .add_enabled(
                    !self.video_path.is_empty() && !self.clips.is_empty(),
                    generate_button,
                )
                .clicked()
            {
                self.generate_clips();
            }
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_app(ctx);
    }
}

fn close_on_esc(ctx: &egui::Context) {
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

fn select_file() -> String {
    match rfd::FileDialog::new().pick_file() {
        Some(file) => file.display().to_string(),
        None => "".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;
    use uuid::Uuid;

    const TEST_VIDEO: &str = "videos/Minecraft_1.20生存#1.偏頭.mkv";
    const TEST_SRT: &str = "videos/Minecraft_1.20生存#1.偏頭.zh.srt";
    const TEST_OUTPUT_DIR_BASE: &str = "/tmp/sub2srs_tests";

    // Helper to get absolute path from relative
    fn get_absolute_path(relative_path: &str) -> String {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .join(relative_path)
            .to_str()
            .unwrap()
            .to_string()
    }

    // Helper to create a unique test directory
    fn setup_test_dir() -> PathBuf {
        let test_run_id = Uuid::new_v4().to_string();
        let output_dir = PathBuf::from(TEST_OUTPUT_DIR_BASE).join(test_run_id);
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir).unwrap();
        }
        fs::create_dir_all(&output_dir).unwrap();
        output_dir
    }

    #[test]
    fn test_srt_parsing_and_conversion() {
        let srt_path = get_absolute_path(TEST_SRT);
        let items = srtparse::from_file(&srt_path).expect("Failed to parse SRT file");
        assert!(!items.is_empty(), "Parsed items should not be empty");

        let clips = convert_subs_to_clips(&items);
        assert_eq!(
            items.len(),
            clips.len(),
            "Clips vector should have the same length as items vector"
        );

        // Check the first clip's data based on the new file
        assert_eq!(clips[0].index, 1);
        assert_eq!(clips[0].start_time, Duration::from_millis(0));
        assert_eq!(clips[0].end_time, Duration::from_millis(2833));
    }

    #[test]
    fn test_ffmpeg_execution() {
        let output_dir = setup_test_dir();
        let output_dir_str = output_dir.to_str().unwrap();
        let srt_path = get_absolute_path(TEST_SRT);
        let video_path = get_absolute_path(TEST_VIDEO);

        let items = srtparse::from_file(&srt_path).expect("Failed to parse SRT file");
        let clips = convert_subs_to_clips(&items);
        assert!(clips.len() > 0, "No clips were parsed from the SRT file.");

        // Test only the first 3 clips to save time
        for clip in clips.iter().take(3) {
            process_clip(&video_path, &output_dir_str.to_string(), clip);
        }

        let output_files = fs::read_dir(output_dir).unwrap().count();
        assert_eq!(output_files, 3 * 2, "Should be two files per clip tested");
    }

    #[test]
    fn test_esc_closes_window() {
        let ctx = egui::Context::default();

        let _ = ctx.run(Default::default(), |ctx| {
            ctx.input_mut(|c| {
                c.events.push(egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: Some(egui::Key::Escape),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                })
            });

            let mut app = MyApp::default();
            app.render_app(ctx);

            let viewport_events = ctx.viewport(|v| v.commands.clone());
            assert!(viewport_events.contains(&egui::ViewportCommand::Close));
        });
    }
}
