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
mod video_to_text;

use clap::Parser;
use teloxide::{requests::Requester, types::ChatId};

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
    println!("Configuration loaded");

    let (req_sender, mut req_receiver) = tokio::sync::mpsc::channel::<telegram::UserRequest>(1);

    let video_bot = telegram::VideoBot::new(
        config.telegram_bot_key.clone(),
        config.telegram_user_secret.clone(),
        req_sender,
    );

    let process_config = config.clone();
    let bot = video_bot.clone();
    tokio::spawn(async move {
        loop {
            if let Some(req) = req_receiver.recv().await {
                let (log_sender, mut log_receiver) = tokio::sync::mpsc::channel::<String>(10);
                let process = video_to_text::video_to_text(
                    &process_config,
                    video_to_text::Request { uri: req.uri },
                    log_sender,
                );

                tokio::pin!(process);
                loop {
                    let msg: Option<String>;
                    let done;
                    tokio::select! {
                        Some(i) = log_receiver.recv() => {
                            msg = Some(i);
                            done = false;
                        },
                        r = &mut process => {
                            match r {
                                Ok(buf) => {
                                    msg = Some(buf);
                                },
                                Err(e) => {
                                    msg = Some(format!("{:?}", e));
                                }
                            }
                            done = true;
                        },
                    };
                    if done {
                        if let Some(msg) = msg {
                            let _ = bot
                                .bot
                                .send_document(
                                    ChatId(req.chat_id),
                                    markdown_to_html::markdown_to_tg(&msg),
                                )
                                .await;
                        }

                        bot.send(req.chat_id, "Finish".to_string()).await;
                        break;
                    }
                    if let Some(msg) = msg {
                        println!("log {}", msg);
                        bot.send(req.chat_id, msg).await;
                    }
                }
            }
        }
    });

    video_bot.start_bot().await;

    Ok(())
}
