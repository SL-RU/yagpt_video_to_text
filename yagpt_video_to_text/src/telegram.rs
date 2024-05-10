use teloxide::{dispatching::dialogue::InMemStorage, prelude::*, requests::Requester};
use tokio::sync::mpsc::Sender;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
type RequestSender = Sender<UserRequest>;
type BotType = Bot;

#[derive(Clone)]
struct SharedData {
    secret_word: String,
}

#[derive(Debug, Clone, Default)]
pub struct UserRequest {
    pub uri: String,
    pub chat_id: i64,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveUrl,
}

#[derive(Clone)]
pub struct VideoBot {
    pub bot: BotType,
    user_request: RequestSender,
    data: SharedData,
}

impl VideoBot {
    pub fn new(key: String, secret_word: String, user_request: Sender<UserRequest>) -> Self {
        Self {
            user_request,
            bot: Bot::new(key),
            data: SharedData { secret_word },
        }
    }

    pub async fn send(&self, chat_id: i64, msg: String) {
        if let Err(e) = self.bot.send_message(ChatId(chat_id), msg).await {
            log::error!("Bot send {}", e);
        }
    }

    pub async fn start_bot(&self) {
        log::info!("Starting dialogue bot...");

        let handler = Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::ReceiveUrl].endpoint(receive_url));

        let mut dispatcher = Dispatcher::builder(self.bot.clone(), handler)
            .dependencies(dptree::deps![
                InMemStorage::<State>::new(),
                self.user_request.clone(),
                self.data.clone()
            ])
            .enable_ctrlc_handler()
            .build();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = dispatcher.dispatch() => {},
        }
    }
}

async fn start(
    bot: BotType,
    dialogue: MyDialogue,
    msg: Message,
    data: SharedData,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            if text == data.secret_word {
                dialogue.update(State::ReceiveUrl).await?;
                bot.send_message(msg.chat.id, "Введите *URL* видео").await?;
            } else {
                bot.send_message(msg.chat.id, "Нужно секретное слово")
                    .await?;
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Введите текст").await?;
        }
    }

    Ok(())
}

async fn receive_url(bot: BotType, msg: Message, s: RequestSender) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let request_result = s.try_send(UserRequest {
                uri: text.to_string(),
                chat_id: msg.chat.id.0,
            });
            match request_result {
                Ok(_) => bot.send_message(msg.chat.id, "Принято").await?,
                Err(_) => {
                    bot.send_message(msg.chat.id, "Очередь занята, подождите")
                        .await?
                }
            };
        }
        None => {
            bot.send_message(msg.chat.id, "Нужна ссылка").await?;
        }
    }

    Ok(())
}
