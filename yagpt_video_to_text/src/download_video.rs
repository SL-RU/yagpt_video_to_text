use std::{
    fs, io,
    path::{Path, PathBuf},
};

use youtube_dl::YoutubeDl;

pub struct VideoData {
    pub path: PathBuf,
    pub name: String,
}

pub async fn download_video(url: String, local_path: &Path) -> io::Result<VideoData> {
    if local_path.exists() {
        fs::remove_file(local_path)?;
    }
    let filename = local_path
        .file_name()
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            format!("filename error {:?}", local_path),
        ))?
        .to_string_lossy();

    let dir = local_path.parent().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        format!("file parent error {:?}", local_path),
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

    let title = data
        .into_single_video()
        .unwrap_or_default()
        .title
        .unwrap_or_default();
    log::info!("Video title: {title:?}");

    output.download_to_async(dir).await.map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotConnected,
            format!("video download {:?}", e),
        )
    })?;

    Ok(VideoData {
        path: local_path.to_path_buf(),
        name: title,
    })
}
