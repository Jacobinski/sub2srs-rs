use eframe::egui;
use std::fs;
use std::path::Path;
use std::process::Command;

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
pub struct SubtitleClip {
    pub index: usize,
    pub start_time: std::time::Duration,
    pub end_time: std::time::Duration,
}

#[derive(Default)]
pub struct MyApp {
    pub video_path: String,
    pub subtitle_path: String,
    pub clips: Vec<SubtitleClip>,
}

pub fn convert_subs_to_clips(subs: &[srtparse::Item]) -> Vec<SubtitleClip> {
    subs.iter()
        .map(|sub| SubtitleClip {
            index: sub.pos,
            start_time: sub.start_time.into_duration(),
            end_time: sub.end_time.into_duration(),
        })
        .collect()
}

pub mod ffmpeg_command {
    // Constants for FFmpeg flags
    pub const INPUT: &str = "-i";
    pub const SEEK: &str = "-ss";
    pub const VFRAMES: &str = "-vframes";
    pub const VF: &str = "-vf";
    pub const SCALE: &str = "scale=-1:320";
    pub const AN: &str = "-an";
    pub const TO: &str = "-to";
    pub const VN: &str = "-vn";
    pub const CODEC_AUDIO: &str = "-c:a";
    pub const LIBMP3LAME: &str = "libmp3lame";
    pub const BITRATE_AUDIO: &str = "-b:a";
    pub const BITRATE_192K: &str = "192k";

    pub fn build_ffmpeg_args_for_clip(
        clip_index: usize,
        clip_start_time: f64,
        clip_end_time: f64,
        video_path: &str,
        output_dir: &str,
    ) -> Vec<String> {
        let start_time_str = clip_start_time.to_string();
        let end_time_str = clip_end_time.to_string();

        vec![
            INPUT.to_string(),
            video_path.to_string(),

            // Screenshot
            SEEK.to_string(),
            start_time_str.clone(),
            VFRAMES.to_string(),
            "1".to_string(),
            VF.to_string(),
            SCALE.to_string(),
            AN.to_string(),
            format!("{}/screenshot_{}.png", output_dir, clip_index),

            // Audio clip
            SEEK.to_string(),
            start_time_str,
            TO.to_string(),
            end_time_str,
            VN.to_string(),
            CODEC_AUDIO.to_string(),
            LIBMP3LAME.to_string(),
            BITRATE_AUDIO.to_string(),
            BITRATE_192K.to_string(),
            format!("{}/audio_clip_{}.mp3", output_dir, clip_index),
        ]
    }
}

impl MyApp {
    fn generate_clips(&self) {
        let output_dir = "/tmp/sub2srs_test"; // This should probably be a constant or configurable
        if !Path::new(output_dir).exists() {
            fs::create_dir_all(output_dir).expect("Failed to create output directory");
        }

        for clip in &self.clips {
            let args = ffmpeg_command::build_ffmpeg_args_for_clip(
                clip.index,
                clip.start_time.as_secs_f64(),
                clip.end_time.as_secs_f64(),
                &self.video_path,
                output_dir,
            );
            println!("Executing ffmpeg for clip {}: {:?}", clip.index, args);

            let status = Command::new("ffmpeg")
                .args(args)
                .status()
                .expect("Failed to execute ffmpeg");

            if !status.success() {
                println!(
                    "ffmpeg command for clip {} failed with status: {}",
                    clip.index, status
                );
            } else {
                println!("Successfully processed clip {}", clip.index);
            }
        }
        println!("All clips processed.");
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(egui::RichText::new("Files").heading());
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

            ui.label(egui::RichText::new("Subtitles").heading());
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

            ui.separator();

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
    fn test_ffmpeg_command_generation() {
        let clip = SubtitleClip {
            index: 1,
            start_time: Duration::from_secs(10),
            end_time: Duration::from_secs(15),
        };
        // Use a fixed path for command generation testing for consistent asserts
        let output_dir = "/tmp/sub2srs_test";
        let video_path = get_absolute_path(TEST_VIDEO);

        let args = ffmpeg_command::build_ffmpeg_args_for_clip(
            clip.index,
            clip.start_time.as_secs_f64(),
            clip.end_time.as_secs_f64(),
            &video_path,
            output_dir,
        );

        let expected_args: Vec<String> = [
            ffmpeg_command::INPUT,
            &video_path,
            ffmpeg_command::SEEK,
            "10",
            ffmpeg_command::VFRAMES,
            "1",
            ffmpeg_command::VF,
            ffmpeg_command::SCALE,
            ffmpeg_command::AN,
            "/tmp/sub2srs_test/screenshot_1.png",
            ffmpeg_command::SEEK,
            "10",
            ffmpeg_command::TO,
            "15",
            ffmpeg_command::VN,
            ffmpeg_command::CODEC_AUDIO,
            ffmpeg_command::LIBMP3LAME,
            ffmpeg_command::BITRATE_AUDIO,
            ffmpeg_command::BITRATE_192K,
            "/tmp/sub2srs_test/audio_clip_1.mp3",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        assert_eq!(args, expected_args);
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
            let args = ffmpeg_command::build_ffmpeg_args_for_clip(
                clip.index,
                clip.start_time.as_secs_f64(),
                clip.end_time.as_secs_f64(),
                &video_path,
                output_dir_str,
            );

            let status = Command::new("ffmpeg")
                .args(args)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .expect("Failed to execute ffmpeg");

            assert!(
                status.success(),
                "ffmpeg command for clip {} should succeed",
                clip.index
            );
        }

        let output_files = fs::read_dir(output_dir).unwrap().count();
        assert_eq!(output_files, 3 * 2, "Should be two files per clip tested");
    }
}
