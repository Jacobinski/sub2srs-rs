use crate::ffmpeg::FFmpegBuilder;
use std::process::Command;

pub fn take_screenshot(time: f64, input: String, output: String) {
    assert!(time >= 0.0);
    assert_ne!(input, "");
    assert_ne!(output, "");

    let ffmpeg = FFmpegBuilder::new(input, output)
        .seek_to(time)
        .output_frames_count(1)
        .scale(320)
        .disable_audio()
        .build();

    Command::new("ffmpeg")
        .args(ffmpeg.args())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("screenshot command should succeed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    const TEST_VIDEO: &str = "videos/Minecraft_1.20生存#1.偏頭.mkv";
    const TEST_OUTPUT_DIR_BASE: &str = "/tmp/sub2srs_tests";

    // Helper to get absolute path from relative
    fn get_absolute_path(relative_path: &str) -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
    fn test_take_screenshot() {
        let time = 10.23;
        let input = get_absolute_path(TEST_VIDEO);
        let output = format!("{}/screenshot.png", setup_test_dir().to_str().unwrap());
        let output_path = PathBuf::from(&output);

        assert!(!output_path.exists());
        take_screenshot(time, input, output);
        assert!(output_path.exists());
    }
}
