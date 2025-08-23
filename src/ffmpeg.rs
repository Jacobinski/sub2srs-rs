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

/// FFmpeg runs an `ffmpeg` CLI command.
pub struct FFmpeg {}

/// FFmpegBuilder builds an FFmpeg struct.
pub struct FFmpegBuilder {
    // Required arguments
    input_path: String,
    output_path: String,
    // Optional arguments
    seek_time: Option<f64>,
}

impl FFmpegBuilder {
    pub fn new(input_path: String, output_path: String) -> Self {
        FFmpegBuilder {
            input_path: input_path,
            output_path: output_path,
            seek_time: None,
        }
    }

    // Starts the input at `time`. Equivalent to the FFmpeg `-ss` flag.
    pub fn seek_to(mut self, time: f64) -> Self {
        self.seek_time = Some(time);
        self
    }
}

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

#[cfg(test)]
mod tests {
    use super::FFmpegBuilder;

    const INPUT: &str = "/directory/input.mp4";
    const OUTPUT: &str = "/directory/output.png";

    #[test]
    fn test_ffmpeg_builder_new() {
        let builder = FFmpegBuilder::new(INPUT.into(), OUTPUT.into());

        assert_eq!(builder.input_path, INPUT);
        assert_eq!(builder.output_path, OUTPUT);
        assert_eq!(builder.seek_time, None);
    }

    #[test]
    fn test_ffmpeg_builder_seek() {
        let seek_time: f64 = 1234.0;
        let builder = FFmpegBuilder::new(INPUT.into(), OUTPUT.into()).seek_to(seek_time);

        assert_eq!(builder.input_path, INPUT);
        assert_eq!(builder.output_path, OUTPUT);
        assert_eq!(builder.seek_time, Some(seek_time));
    }
}
