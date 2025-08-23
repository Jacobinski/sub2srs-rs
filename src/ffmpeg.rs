use ez_ffmpeg::{FfmpegContext, Output};

pub fn create_screenshot(
    video_path: &str,
    screenshot_time: f64,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time_us = (screenshot_time * 1_000_000.0) as i64;

    FfmpegContext::builder()
        .input(video_path)
        .output(
            Output::new(output_path)
                .set_start_time_us(start_time_us)
                .set_max_video_frames(1),
        )
        .filter_desc("scale=-1:320")
        .build()?
        .start()?
        .wait()?;

    Ok(())
}

pub fn create_audio_clip(
    video_path: &str,
    clip_start_time: f64,
    clip_end_time: f64,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time_us = (clip_start_time * 1_000_000.0) as i64;
    let recording_time_us = ((clip_end_time - clip_start_time) * 1_000_000.0) as i64;
    FfmpegContext::builder()
        .input(video_path)
        .output(
            Output::new(output_path)
                .set_start_time_us(start_time_us)
                .set_recording_time_us(recording_time_us)
                .set_audio_codec("libmp3lame")
                .set_audio_codec_opt("b", "192k")
                .add_stream_map("0:a"), // Map audio stream, excluding video
        )
        .build()?
        .start()?
        .wait()?;
    Ok(())
}
