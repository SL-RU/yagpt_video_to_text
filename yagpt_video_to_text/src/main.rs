mod convert_video_to_audio;
mod download_video;
mod iam_generator;
mod speech_to_text;
mod upload;
mod api {
    tonic::include_proto!("yandex");
}
mod cloud_operation;
mod config;
mod gpt_processor;

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use convert_video_to_audio::convert_video_to_audio;
use encoding::{self, Encoding};
use iam_generator::IAMGenerator;
use tokio::io::AsyncWriteExt;
use upload::Uploader;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
    uri: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = config::Config::read(&args.config);

    println!("Configuration loaded");
    println!("Downloading video");
    let video_path =
        download_video::download_video(args.uri, Path::new(&config.video_path)).await?;

    println!("Download complete");
    println!("Converting video to audio");
    let audio_path =
        convert_video_to_audio(video_path, PathBuf::from_str(&config.audio_path)?).await?;
    println!("Converting complete");

    println!("Uploading audio to S3");
    let uploader = Uploader::new(&config);
    let bucket_object_uri = uploader.upload(audio_path).await?;
    println!("Uploading complete");

    println!("Generating IAM token");
    let client = IAMGenerator::new(config.token_path.clone())?;
    let token = client.generate_iam_token().await?;

    println!("Audio to text. It may be long!");
    let text_list = speech_to_text::autio_to_text(&token, bucket_object_uri).await?;

    println!("Audio to text complete. Lines count: {}", text_list.len());

    let mut file = tokio::fs::File::create(config.transcribed_text.clone()).await?;
    for line in text_list.iter() {
        let line = encoding::all::UTF_8.encode(line, encoding::EncoderTrap::Replace)?;
        file.write_all(&line).await?;
        file.write_all(b"\r\n").await?;
    }
    file.shutdown().await?;

    let mut process_text = String::new();
    let mut chars = 0;
    let mut file = tokio::fs::File::create(config.refactored_md.clone()).await?;
    for (index, line) in text_list.iter().enumerate() {
        process_text += line;
        process_text += "\r\n";

        const MAX_PROMT_SIZE: usize = 10000;
        if process_text.len() > MAX_PROMT_SIZE {
            chars += process_text.len();
            let res = gpt_processor::proceed(&token, &config, process_text).await?;
            println!("Gpt processor [{}] {}/{}", chars, index, text_list.len());
            let line = encoding::all::UTF_8.encode(&res, encoding::EncoderTrap::Replace)?;
            file.write_all(&line).await?;
            file.write_all(b"\r\n\r\n").await?;
            process_text = String::new();
        }
    }

    if !process_text.is_empty() {
        chars += process_text.len();
        let res = gpt_processor::proceed(&token, &config, process_text).await?;
        let line = encoding::all::UTF_8.encode(&res, encoding::EncoderTrap::Replace)?;
        file.write_all(&line).await?;
    }

    file.shutdown().await?;
    println!("Gpt processor [{}] {}", chars, text_list.len());
    println!("Final file: {}", config.refactored_md);

    Ok(())
}
