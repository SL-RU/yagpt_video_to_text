use std::io::Error;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, io};

pub async fn convert_video_to_audio(
    video_path: PathBuf,
    audio_path: PathBuf,
) -> io::Result<PathBuf> {
    let video_path = video_path.canonicalize()?;
    let video_path_str = video_path.to_str().ok_or(io::Error::new(
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

    let res = Command::new("ffmpeg")
        .arg("-y")
        .args(vec!["-i", video_path_str])
        .args(vec!["-ac", "1"])
        .args(vec!["-ab", "48k"])
        .arg(audio_path_str)
        .output()?;

    if !res.status.success() {
        return Err(Error::new(io::ErrorKind::Other, format!("'{:?}", res)));
    }

    Ok(audio_path)
}
