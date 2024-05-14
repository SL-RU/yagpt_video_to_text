use std::path::PathBuf;

use crate::{
    config::Config,
    markdown_to_html,
    telegram::{UserRequest, VideoBot},
    video_to_text::{self, video_to_text},
};
use teloxide::{requests::Requester, types::ChatId};
use tokio::sync::mpsc::Receiver;

enum RequestInfo {
    Message(String),
    Done(String),
    Error(String),
}

async fn execute_request(config: &Config, bot: &VideoBot, req: UserRequest) {
    let (log_sender, mut log_receiver) = tokio::sync::mpsc::channel::<String>(10);
    let process = video_to_text(config, video_to_text::Request { uri: req.uri }, log_sender);

    tokio::pin!(process);
    loop {
        let msg: RequestInfo = tokio::select! {
            _ = tokio::signal::ctrl_c() => return,
            Some(i) = log_receiver.recv() => RequestInfo::Message(i),
            r = &mut process => match r {
                Ok(buf) => {
                    RequestInfo::Done(buf)
                },
                Err(e) => {
                    RequestInfo::Error(format!("{:?}", e))
                }
            },
        };
        match msg {
            RequestInfo::Message(msg) => bot.send(req.chat_id, msg).await,
            RequestInfo::Done(data) => {
                let _ = bot
                    .bot
                    .send_document(
                        ChatId(req.chat_id),
                        markdown_to_html::markdown_to_tg(config, data),
                    )
                    .await;
                bot.send(req.chat_id, "Finish".to_string()).await;
                return;
            }
            RequestInfo::Error(err) => {
                bot.send(req.chat_id, format!("ERROR: {:?}", err)).await;
                return;
            }
        };
    }
}

pub async fn start_executor(
    config: Config,
    bot: VideoBot,
    mut req_receiver: Receiver<UserRequest>,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => return,
                Some(req) = req_receiver.recv() => execute_request(&config, &bot, req).await,
            }
        }
    });
}
