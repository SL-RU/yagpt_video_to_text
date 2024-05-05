mod convert_video_to_audio;
mod download_video;
mod iam_generator;
mod speech_to_text;
mod upload;
mod api {
    tonic::include_proto!("yandex");
}
mod config;

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use convert_video_to_audio::convert_video_to_audio;
use iam_generator::IAMGenerator;

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

    let video_path = download_video::download_video(args.uri, Path::new(&config.video_path))
        .await
        .unwrap();

    let audio_path =
        convert_video_to_audio(video_path, PathBuf::from_str(&config.audio_path).unwrap())
            .await
            .unwrap();

    let bucket_object_uri = upload::upload(audio_path).await.unwrap();

    let client = IAMGenerator::new(config.token_path).unwrap();

    let token = client.generate_iam_token().await.unwrap();

    speech_to_text::autio_to_text(&token, bucket_object_uri)
        .await
        .unwrap();
}
