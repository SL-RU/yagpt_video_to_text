use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

pub async fn convert_video_to_audio(
    video_path: PathBuf,
    audio_path: PathBuf,
) -> io::Result<PathBuf> {
    let path_str = video_path.to_str().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Input path error {:?}", video_path),
    ))?;

    if audio_path.exists() {
        fs::remove_file(audio_path.clone())?;
    }
    let audio_path_str = audio_path.to_str().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Output path error {:?}", audio_path),
    ))?;

    Command::new("ffmpeg")
        .arg("-y")
        .args(vec!["-i", path_str])
        .args(vec!["-ac", "1"])
        .args(vec!["-ab", "48k"])
        .args(vec!["-codec:a", "libmp3lame"])
        .arg(audio_path_str)
        .output()?;

    Ok(audio_path)
}
