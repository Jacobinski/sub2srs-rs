use crate::ffmpeg::FFmpegBuilder;
use std::process::Command;

pub fn record_audio_clip(start_time: f64, end_time: f64, input: String, output: String) {
    assert!(
        Command::new("ffmpeg")
            .args(["-version"])
            .status()
            .expect("failed to execute ffmpeg -version")
            .success()
    );

    let ffmpeg = FFmpegBuilder::new(input, output)
        .seek_to(start_time)
        .end_at(end_time)
        .disable_video()
        .encode_mp3_audio()
        .build();

    Command::new("ffmpeg")
        .args(ffmpeg.args())
        .status()
        .expect("audio clip command should succeed");
}
