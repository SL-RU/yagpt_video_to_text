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
use iam_generator::IAMGenerator;
use upload::Uploader;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
    uri: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = config::Config::read(&args.config);

    println!("Configuration loaded");
    println!("Downloading video");
    let video_path = download_video::download_video(args.uri, Path::new(&config.video_path))
        .await
        .unwrap();

    println!("Download complete");
    println!("Converting video to audio");
    let audio_path =
        convert_video_to_audio(video_path, PathBuf::from_str(&config.audio_path).unwrap())
            .await
            .unwrap();
    println!("Converting complete");

    println!("Uploading audio to S3");
    let uploader = Uploader::new(&config);
    let bucket_object_uri = uploader.upload(audio_path).await.unwrap();
    println!("Uploading complete");

    println!("Generating IAM token");
    let client = IAMGenerator::new(config.token_path.clone()).unwrap();
    let token = client.generate_iam_token().await.unwrap();

    println!("Audio to text. It may be long!");
    let text_list = speech_to_text::autio_to_text(&token, bucket_object_uri)
        .await
        .unwrap();

    println!("Audio to text complete. Lines count: {}", text_list.len());

    let mut process_text = String::new();
    let mut output = String::new();
    for (index, line) in text_list.iter().enumerate() {
        process_text += line;
        process_text += "\r\n";

        if process_text.len() > 1000 {
            let res = gpt_processor::proceed(&token, &config, process_text)
                .await
                .unwrap();
            println!("Gpt processor {}/{}", index, text_list.len());
            output += &res;
            output += "\r\n";
            process_text = String::new();
        }
    }

    if !process_text.is_empty() {
        let res = gpt_processor::proceed(&token, &config, process_text)
            .await
            .unwrap();
        output += &res;
        output += "\r\n";
    }

    println!("Gpt processor done: {}", output);
}
