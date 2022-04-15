use crate::model;
use crate::service::Context;
use crate::transport::{Telegram, Transport};
use anyhow::{anyhow, Result};
use async_std::task;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Default, Debug, Serialize)]
struct GetUpdates {
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_updates: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct Update {
    pub update_id: i32,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i32,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub message_id: i32,
    pub chat: Chat,
    pub text: Option<String>,
}

#[derive(Deserialize)]
struct ResponsePayload {
    ok: bool,
    description: Option<String>,
    result: Option<Vec<Update>>,
}

struct Poller {
    uri: String,
    ctx: Context,
    offset: i32,
    tg_transport: Telegram,
}

impl Poller {
    fn new(ctx: Context) -> Self {
        let base_url = &ctx.conf.telegram.url;
        let access_token = &ctx.conf.telegram.token;
        let uri = format!("{}bot{}/getUpdates", base_url, access_token);
        let tg_transport = Telegram::new(base_url.as_str(), access_token.as_str());

        Poller {
            ctx,
            uri,
            offset: 0,
            tg_transport,
        }
    }

    async fn get_updates(&mut self) -> Result<Vec<Update>> {
        let mut data = GetUpdates::default();
        data.limit = Some(100);
        data.timeout = Some(5);
        data.allowed_updates = Some(vec![String::from("message")]);

        if self.offset > 0 {
            data.offset = Some(self.offset);
        }

        log::info!("[TelegramPoller] {:?}", data);

        let mut res = match surf::post(&self.uri).body_json(&data) {
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
            }) => Err(anyhow!(description)),
            Ok(response) => match response.result {
                None => Err(anyhow!("invalid result")),
                Some(updates) => {
                    if !updates.is_empty() {
                        self.offset = updates.last().unwrap().update_id + 1;
                    }

                    Ok(updates)
                }
            },
            Err(err) => Err(err.into_inner()),
        };
    }

    async fn handle_message(&self, message: &Message) -> Result<()> {
        if message.text.is_none() {
            return Ok(());
        }

        let text = message.text.as_ref().unwrap();
        if !text.starts_with("/start ") {
            return Ok(());
        }

        let open_id = &text["/start ".len()..];
        let chat_id = message.chat.id.to_string();
        let user = self.ctx.user_model.find_one_by_open_id(open_id).await?;

        let result = self
            .ctx
            .transport_model
            .find_one_by_user_id_type(user.id, model::transport_type::TELEGRAM)
            .await;
        match result {
            Ok(_) => {
                self.ctx
                    .transport_model
                    .update_chat_id(user.id, model::transport_type::TELEGRAM, chat_id.as_str())
                    .await?;
            }
            Err(err) => match model::is_not_found_record_err(&err) {
                true => {
                    let mut transport =
                        model::Transport::new(user.id, model::transport_type::TELEGRAM);
                    transport.connected = true;
                    transport.chat_id = Some(message.chat.id.to_string());
                    self.ctx.transport_model.insert(&transport).await?;
                }
                false => return Ok(()),
            },
        }

        self.tg_transport.push(
                chat_id.as_str(),
                "",
                "Your Telegram transport has been configured. You can now receive notifications from simple push service.",
        ).await?;

        Ok(())
    }

    async fn start_polling(&mut self, running: &AtomicBool) {
        running.store(true, Ordering::Release);

        log::info!("[TelegramPoller] start polling");

        while running.load(Ordering::Acquire) {
            let updates = match self.get_updates().await {
                Ok(updates) => updates,
                Err(err) => {
                    log::error!("[TelegramPoller] failed to getUpdates, {}", err.to_string());
                    continue;
                }
            };

            for update in &updates {
                if update.message.is_none() {
                    continue;
                }
                match self.handle_message(update.message.as_ref().unwrap()).await {
                    Ok(_) => {}
                    Err(_) => {}
                };
            }
        }

        log::info!("[TelegramPoller] stop polling");
    }
}

pub struct TelegramBot {
    ctx: Context,
    state: Arc<AtomicBool>,
}

impl Drop for TelegramBot {
    fn drop(&mut self) {
        self.state.store(false, Ordering::Release);
    }
}

impl TelegramBot {
    pub fn new(ctx: Context) -> Self {
        TelegramBot {
            ctx,
            state: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> Result<()> {
        let ctx = self.ctx.clone();
        let state = self.state.clone();
        if let Err(_) = state.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) {
            return Err(anyhow!("already running"));
        }

        task::spawn(async move {
            let mut poller = Poller::new(ctx);
            poller.start_polling(state.as_ref()).await;
        });

        Ok(())
    }
}
