use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::config::Config;
use crate::convert_video_to_audio::convert_video_to_audio;
use crate::iam_generator::IAMGenerator;
use crate::upload::Uploader;

#[derive(Debug, Clone)]
pub struct Request {
    pub uri: String,
}

pub async fn video_to_text(
    config: &Config,
    request: Request,
    channel: tokio::sync::mpsc::Sender<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let log = |msg: String| {
        let channel = channel.clone();
        log::info!("{}", msg);
        tokio::spawn(async move {
            channel
                .send(msg)
                .await
                .unwrap_or_else(|err| log::error!("video_to_text channel send error {:?}", err))
        });
    };
    let log_str = |msg: &str| {
        log(msg.to_string());
    };

    log_str("Downloading video");
    let video =
        crate::download_video::download_video(request.uri, Path::new(&config.video_path)).await?;
    let video_path = video.path;

    log_str(&format!("Download complete: {:?}", video.name));
    log_str("Converting video to audio");

    let audio_path =
        convert_video_to_audio(video_path, PathBuf::from_str(&config.audio_path)?).await?;
    log_str("Converting complete");

    log_str("Uploading audio to S3");
    let uploader = Uploader::new(config);
    let bucket_object_uri = uploader.upload(audio_path).await?;
    log_str("Uploading complete");

    log_str("Generating token");
    let client = IAMGenerator::new(config.token_path.clone())?;
    log_str("Generating\\.\\.\\.");
    let token = client.generate_iam_token().await?;

    log_str("Audio to text, it may be long!");
    let text_list = crate::speech_to_text::autio_to_text(&token, bucket_object_uri).await?;

    log(format!(
        "Audio to text complete. Lines count: {}",
        text_list.len()
    ));

    let mut process_text = String::new();
    let mut chars = 0;
    let mut output = String::new();
    log_str("Starting");
    let mut gpt = crate::gpt_processor::GptProcessor::new(&token, config).await?;
    log_str("Started gpt");
    for (index, line) in text_list.iter().enumerate() {
        process_text += line;
        process_text += "\r\n";

        const MAX_PROMT_SIZE: usize = 5000;
        if process_text.len() > MAX_PROMT_SIZE {
            chars += process_text.len();
            let res = gpt.proceed(process_text).await?;
            log(format!(
                "Gpt processor [{}] {}/{}",
                chars,
                index,
                text_list.len()
            ));
            output += &res;
            output += "\r\n\r\n";
            process_text = String::new();
        }
    }

    if !process_text.is_empty() {
        chars += process_text.len();
        let res = gpt.proceed(process_text).await?;
        output += &res;
    }

    log(format!("Gpt processor [{}] {}", chars, text_list.len()));
    Ok(output)
}
