use crate::ffmpeg::FFmpegBuilder;
use std::process::{ExitStatus, Output};
use tokio::process::Command;

pub async fn record_audio_clip(
    start_time: f64,
    end_time: f64,
    input: String,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(start_time >= 0.0);
    assert!(end_time > start_time);
    assert_ne!(input, "");
    assert_ne!(output, "");

    let ffmpeg = FFmpegBuilder::new(input, output)
        .seek_to(start_time)
        .end_at(end_time)
        .disable_video()
        .encode_mp3_audio()
        .build();

    Command::new("ffmpeg")
        .args(ffmpeg.args())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    const TEST_VIDEO: &str = "videos/Minecraft_1.20生存#1.偏頭.mkv";
    const TEST_OUTPUT_DIR_BASE: &str = "/tmp/subs2srs_tests";

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

    #[tokio::test]
    async fn test_record_audio() {
        let start_time = 10.10;
        let end_time = 12.12;
        let input = get_absolute_path(TEST_VIDEO);
        let output = format!("{}/audio_clip.mp3", setup_test_dir().to_str().unwrap());
        let output_path = PathBuf::from(&output);

        assert!(!output_path.exists());
        record_audio_clip(start_time, end_time, input, output)
            .await
            .expect("failed to record audio clip");
        assert!(output_path.exists());
    }
}
