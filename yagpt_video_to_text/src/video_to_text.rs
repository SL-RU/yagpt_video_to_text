use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::convert_video_to_audio::convert_video_to_audio;
use crate::iam_generator::IAMGenerator;
use crate::upload::Uploader;
use encoding::{self, Encoding};
use tokio::io::AsyncWriteExt;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Request {
    pub uri: String,
}

pub async fn video_to_text(
    config: &Config,
    request: Request,
    channel: tokio::sync::mpsc::Sender<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    channel.send("Downloading video".to_string()).await?;
    let video_path =
        crate::download_video::download_video(request.uri, Path::new(&config.video_path)).await?;

    channel.send("Download complete".to_string()).await?;
    channel
        .send("Converting video to audio".to_string())
        .await?;

    let audio_path =
        convert_video_to_audio(video_path, PathBuf::from_str(&config.audio_path)?).await?;
    channel.send("Converting complete".to_string()).await?;

    channel.send("Uploading audio to S3".to_string()).await?;
    let uploader = Uploader::new(config);
    let bucket_object_uri = uploader.upload(audio_path).await?;
    channel.send("Uploading complete".to_string()).await?;

    channel.send("Generating token".to_string()).await?;
    let client = IAMGenerator::new(config.token_path.clone())?;
    channel.send("Generating\\.\\.\\.".to_string()).await?;
    let token = client.generate_iam_token().await?;

    channel
        .send("Audio to text, it may be long!".to_string())
        .await?;
    let text_list = crate::speech_to_text::autio_to_text(&token, bucket_object_uri).await?;

    channel
        .send(format!(
            "Audio to text complete. Lines count: {}",
            text_list.len()
        ))
        .await?;

    {
        let mut file = tokio::fs::File::create(config.transcribed_text.clone()).await?;
        for line in text_list.iter() {
            let line = encoding::all::UTF_8.encode(line, encoding::EncoderTrap::Replace)?;
            file.write_all(&line).await?;
            file.write_all(b"\r\n").await?;
        }
        file.shutdown().await?;
    }

    let mut process_text = String::new();
    let mut chars = 0;
    let mut output = String::new();
    channel.send("Starting".to_string()).await?;
    let mut gpt = crate::gpt_processor::GptProcessor::new(&token, config).await?;
    channel.send("Started gpt".to_string()).await?;
    for (index, line) in text_list.iter().enumerate() {
        process_text += line;
        process_text += "\r\n";

        const MAX_PROMT_SIZE: usize = 5000;
        if process_text.len() > MAX_PROMT_SIZE {
            chars += process_text.len();
            let res = gpt.proceed(process_text).await?;
            channel
                .send(format!(
                    "Gpt processor [{}] {}/{}",
                    chars,
                    index,
                    text_list.len()
                ))
                .await?;
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

    channel
        .send(format!("Gpt processor [{}] {}", chars, text_list.len()))
        .await?;

    Ok(output)
}
