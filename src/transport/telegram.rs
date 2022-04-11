use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub const TELEGRAM_API_ENDPOINT: &str = "https://api.telegram.org";

pub struct Telegram {
    uri: String,
}

impl Telegram {
    pub fn new(access_token: &str) -> Self {
        Telegram {
            uri: format!("{}/bot{}/sendMessage", TELEGRAM_API_ENDPOINT, access_token),
        }
    }
}

#[derive(Serialize)]
struct SendMessage {
    chat_id: String,
    text: String,
    parse_mode: String,
}

#[derive(Deserialize)]
struct ResponsePayload {
    ok: bool,
    description: Option<String>,
}

#[async_trait]
impl crate::transport::Transport for Telegram {
    async fn push(&self, chat: &str, title: &str, message: &str) -> Result<()> {
        let data = &SendMessage {
            chat_id: chat.into(),
            text: format!("\\[*{}*\\]\n\n{}", title, message),
            parse_mode: "MarkdownV2".into(),
        };

        let mut res = match surf::post(&self.uri).body_json(data) {
            Ok(req) => match req.await {
                Ok(res) => res,
                Err(err) => return Err(err.into_inner()),
            },
            Err(err) => return Err(err.into_inner()),
        };

        return match res.body_json::<ResponsePayload>().await {
            Ok(ResponsePayload {
                ok: false,
                description: Some(description),
                ..
            }) => Err(anyhow::anyhow!(description)),
            Ok(_) => Ok(()),
            Err(err) => Err(err.into_inner()),
        };
    }
}
