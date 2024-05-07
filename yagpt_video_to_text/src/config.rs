use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub video_path: String,
    pub audio_path: String,
    pub transcribed_json: String,
    pub transcribed_text: String,
    pub refactored_md: String,
    pub refactored_html: String,
    pub model_uri: String,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub bucket_name: String,
    pub token_path: String,
    pub telegram_bot_key: String,
    pub telegram_user_secret: String,
}

impl Config {
    pub fn read(path: &String) -> Self {
        let data = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Error reading config {:?}: {:?}", path, e));
        serde_json::from_str(&data).expect("Config parsing error")
    }
}
