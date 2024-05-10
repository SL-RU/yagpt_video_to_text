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
mod markdown_to_html;
mod telegram;
mod telegram_executor;
mod video_to_text;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let args = Args::parse();
    let config = config::Config::read(&args.config);
    log::info!("Configuration loaded");

    let (req_sender, req_receiver) = tokio::sync::mpsc::channel::<telegram::UserRequest>(1);

    let video_bot = telegram::VideoBot::new(
        config.telegram_bot_key.clone(),
        config.telegram_user_secret.clone(),
        req_sender,
    );

    telegram_executor::start_executor(config.clone(), video_bot.clone(), req_receiver).await;

    video_bot.start_bot().await;

    Ok(())
}
