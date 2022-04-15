use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct Telegram {
    uri: String,
}

impl Telegram {
    pub fn new(url: &str, access_token: &str) -> Self {
        Telegram {
            uri: format!("{}bot{}/sendMessage", url, access_token),
        }
    }
}

#[derive(Serialize)]
struct SendMessage {
    chat_id: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<String>,
}

#[derive(Deserialize)]
struct ResponsePayload {
    ok: bool,
    description: Option<String>,
}

#[async_trait]
impl super::Transport for Telegram {
    async fn push(&self, chat: &str, title: &str, message: &str) -> Result<()> {
        let text = if title.is_empty() {
            format!("{}", message)
        } else {
            format!("[{}]\n\n{}", title, message)
        };

        let data = &SendMessage {
            text,
            chat_id: chat.into(),
            parse_mode: None,
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
