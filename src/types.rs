use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUpdatesResponse {
    pub ok: bool,
    pub result: Vec<TelegramUpdate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: TelegramMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: TelegramMessageFrom,
    pub chat: TelegramMessageChat,
    pub date: i64,
    pub text: Option<String>,
    pub sticker: Option<TelegramMessageSticker>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramMessageFrom {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub username: String,
    pub language_code: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramMessageChat {
    pub id: i64,
    pub first_name: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramMessageSticker {
    pub file_id: String,
}
