use std::{
    fs, io,
    path::{Path, PathBuf},
};

use youtube_dl::YoutubeDl;

pub async fn download_video(url: String, path: &Path) -> io::Result<PathBuf> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    let filename = path
        .file_name()
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            format!("filename error {:?}", path),
        ))?
        .to_string_lossy();

    let dir = path.parent().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        format!("file parent error {:?}", path),
    ))?;

    let mut output = YoutubeDl::new(url);
    output
        .socket_timeout("15")
        .output_template(filename)
        .format("m4a/bestaudio/best");

    let data = output.run_async().await.map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotConnected,
            format!("video data fetch {:?}", e),
        )
    })?;

    println!(
        "Video title: {:?}",
        data.into_single_video().unwrap_or_default().title
    );

    output.download_to_async(dir).await.map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotConnected,
            format!("video download {:?}", e),
        )
    })?;

    Ok(path.to_path_buf())
}
