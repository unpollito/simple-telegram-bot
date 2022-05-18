use std::{sync::{Arc, atomic::{AtomicI64, Ordering}}, collections::HashMap};

use tokio::sync::mpsc;
use types::TelegramUpdatesResponse;

pub mod types;
 
#[derive(Clone)]
pub struct SimpleTelegramBot {
    bot_token: String,
    last_update_id: Arc<AtomicI64>,
}

impl SimpleTelegramBot {
    pub fn new(bot_token: String) -> SimpleTelegramBot {
        SimpleTelegramBot {
            bot_token,
            last_update_id: Arc::new(AtomicI64::new(0)),
        }
    }

    pub fn updater(&self) -> SimpleTelegramBotUpdater {
        SimpleTelegramBotUpdater::new(self.get_base_url(), self.last_update_id.clone())
    }

    pub fn sender(&self) -> SimpleTelegramBotSender {
        SimpleTelegramBotSender::new(self.get_base_url())
    }

    fn get_base_url(&self) -> String {
        format!("https://api.telegram.org/bot{}/", self.bot_token)
    }
}

#[derive(Clone)]
pub struct SimpleTelegramBotUpdater {
    base_url: String,
    last_update_id: Arc<AtomicI64>,
}

impl SimpleTelegramBotUpdater {
    fn new(base_url: String, last_update_id: Arc<AtomicI64>) -> SimpleTelegramBotUpdater {
        SimpleTelegramBotUpdater {
            base_url,
            last_update_id,
        }
    }

    pub fn updates(&self) -> mpsc::Receiver<TelegramUpdatesResponse> {
        let (sender, receiver) = mpsc::channel::<TelegramUpdatesResponse>(100);
        let base_url = self.base_url.clone();
        let last_update_id: Arc<AtomicI64> = self.last_update_id.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            loop {
                let url = format!(
                    "{}getUpdates?timeout=20&offset={}",
                    base_url,
                    last_update_id.load(Ordering::Relaxed) + 1
                );
                let req = client.get(url).send().await;
                match req {
                    Err(e) => {
                        log::warn!("Failed to fetch updates: {}", e);
                    }
                    Ok(req) => {
                        let parsed = req.json::<TelegramUpdatesResponse>().await;
                        match parsed {
                            Ok(response) => {
                                for update in &response.result {
                                    last_update_id.fetch_max(update.update_id, Ordering::Relaxed);
                                }
                                if let Err(e) = sender.send(response).await {
                                    log::error!("Failed to send updates: {}", e);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to parse updates: {}", e);
                            }
                        }
                    }
                };
            }
        });
        return receiver;
    }
}

#[derive(Clone)]
pub struct SimpleTelegramBotSender {
    base_url: String,
    client: reqwest::Client,
}

impl SimpleTelegramBotSender {
    fn new(base_url: String) -> SimpleTelegramBotSender {
        SimpleTelegramBotSender {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_message(&self, chat_id: i64, message: &str) {
        let mut body = HashMap::new();
        body.insert("chat_id", chat_id.to_string());
        body.insert("text", String::from(message));
        let url = format!("{}sendMessage", self.base_url);
        let response_result = self.client.post(url).json(&body).send().await;
        handle_maybe_request_failure(response_result, chat_id, "message");
    }

    pub async fn send_sticker(&self, chat_id: i64, sticker_id: &str) {
        let mut body = HashMap::new();
        body.insert("chat_id", chat_id.to_string());
        body.insert("sticker", String::from(sticker_id));
        let url = format!("{}sendSticker", self.base_url);
        let response_result = self.client.post(url).json(&body).send().await;
        handle_maybe_request_failure(response_result, chat_id, "sticker");
    }
}

fn handle_maybe_request_failure(
    response_result: Result<reqwest::Response, reqwest::Error>,
    chat_id: i64,
    request_type: &str,
) -> () {
    if let Err(err) = response_result {
        log::error!("Failed to send message for chat ID {}: {}", chat_id, err);
    } else {
        let response = response_result.unwrap();
        if response.status().is_client_error() || response.status().is_server_error() {
            log::error!(
                "Failed to send {} for chat ID {}, got an status code {}: {:?}",
                request_type,
                chat_id,
                response.status(),
                response
            );
        }
    }
}