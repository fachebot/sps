use crate::model;
use crate::service::Context;
use crate::transport;
use anyhow::{anyhow, Result};
use async_std::{channel, channel::Receiver, channel::Sender, sync::Arc, task};
use rand::{thread_rng, Rng};
use redis::aio::Connection;
use redis::{AsyncCommands, RedisResult};
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};

fn new_transporter(
    ctx: &Arc<Context>,
    name: &str,
) -> Option<Box<dyn transport::Transport + Send + Sync>> {
    return match name {
        model::transport_type::TELEGRAM => Some(Box::new(transport::Telegram::new(
            &ctx.conf.telegram.url,
            &ctx.conf.telegram.token,
        ))),
        _ => None,
    };
}

struct Worker {
    ctx: Arc<Context>,
}

impl Worker {
    fn new(ctx: Arc<Context>) -> Self {
        Worker { ctx }
    }

    async fn run(&self, receiver: Receiver<i64>) {
        let mut conn = self
            .ctx
            .redis_client
            .get_async_std_connection()
            .await
            .unwrap();

        loop {
            let data = receiver.recv().await;
            if let Err(_) = data {
                break;
            }

            let task_id = data.unwrap();
            log::debug!("[Worker] consume, task_id: {}", task_id);

            let task = self.ctx.task_model.find_one_by_id(task_id).await;
            if let Err(err) = task {
                log::error!(
                    "[Worker] failed to find task by id, task_id: {}, reason: {}",
                    task_id,
                    err
                );
                continue;
            }

            self.push(&mut conn, task.unwrap()).await;
        }
    }

    async fn push(&self, conn: &mut Connection, task: model::Task) {
        let message = self.ctx.message_model.find_one_by_id(task.message_id).await;
        if let Err(err) = message {
            self.retry_task(conn, &task, &err.to_string()).await;
            return;
        }

        let transporter = new_transporter(&self.ctx, &task.transport_type);
        if transporter.is_none() {
            self.skip_task(task.id, "transport not found").await;
            return;
        }

        let message = message.unwrap();
        let transporter = transporter.unwrap();

        let result = transporter
            .push(&task.chat_id, &message.title, &message.content)
            .await;
        if let Err(err) = result {
            self.retry_task(conn, &task, &err.to_string()).await;
            return;
        }

        log::debug!(
            "[Worker] message delivered, message_id: {}, transport: {}",
            message.id,
            &task.transport_type
        );

        if let Err(err) = self.ctx.task_model.set_done(task.id).await {
            log::error!(
                "[Worker] failed to set task state as done, task_id: {}, {}",
                task.id,
                err
            )
        }
    }

    async fn skip_task(&self, task_id: i64, reason: &str) {
        log::error!(
            "[Worker] skip task, task_id: {}, reason: {}",
            task_id,
            reason
        );
    }

    async fn retry_task(&self, conn: &mut Connection, task: &model::Task, reason: &str) {
        log::warn!(
            "[Worker] retry task, task_id: {}, reason: {}",
            task.id,
            reason
        );

        let r = thread_rng().gen_range(0..30);
        // Formula taken from https://github.com/mperham/sidekiq.
        let s = task.retry_count.pow(4) + 15 + (r * (task.retry_count + 1));

        let now = chrono::Utc::now().timestamp();
        let key = &self.ctx.conf.redis.queue_name;
        let result = conn
            .zadd::<_, _, _, i32>(key.as_str(), task.id, now + i64::from(s))
            .await;
        if let Err(err) = result {
            log::error!(
                "[Worker] failed to retry task, task_id: {}, reason: {}",
                task.id,
                err
            );
            return;
        }

        let result = self
            .ctx
            .task_model
            .update_retry_state(task.id, reason)
            .await;
        if let Err(err) = result {
            log::error!(
                "[Worker] failed to update retry state, task_id: {}, reason: {}",
                task.id,
                err
            );
            return;
        }
    }
}

struct Poller {
    ctx: Arc<Context>,
    next: usize,
    senders: Vec<Sender<i64>>,
}

impl Poller {
    fn new(ctx: Arc<Context>, workers: u32) -> Self {
        let mut senders = Vec::<Sender<i64>>::new();
        for _ in 0..workers {
            let (sender, receiver) = channel::unbounded();
            senders.push(sender);

            let ctx = ctx.clone();
            task::spawn(async move {
                let worker = Worker::new(ctx);
                worker.run(receiver).await;
            });
        }

        Poller {
            ctx,
            next: 0,
            senders,
        }
    }

    async fn start_polling(&mut self, running: &AtomicBool) {
        running.store(true, Ordering::Release);
        let queue_name = self.ctx.conf.redis.queue_name.clone();

        log::info!("[Pusher] start polling");

        while running.load(Ordering::Acquire) {
            let ts = chrono::Utc::now().timestamp();
            let mut guard = self.ctx.redis_connection.lock().await;

            let result: RedisResult<Vec<i64>> = guard
                .deref_mut()
                .zrangebyscore(queue_name.as_str(), 0, ts)
                .await;
            if let Err(err) = result {
                log::error!("[Pusher] redis read error, {}", err);

                task::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            let task_ids = result.unwrap();
            if task_ids.is_empty() {
                task::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            let result: RedisResult<i64> = guard
                .deref_mut()
                .zrembyscore(queue_name.as_str(), 0, ts)
                .await;
            if let Err(err) = result {
                log::error!("[Pusher] redis write error, {}", err);
            }

            for task_id in task_ids {
                let result = self.senders[self.next].send(task_id).await;
                if let Err(err) = result {
                    log::error!(
                        "[Pusher] failed to assign task, task_id: {}, {}",
                        task_id,
                        err
                    );
                }

                self.next += 1;
                if self.next >= self.senders.len() {
                    self.next = 0;
                }
            }
        }

        for sender in &self.senders {
            sender.close();
        }

        log::info!("[Pusher] stop polling");
    }
}

pub struct Pusher {
    ctx: Arc<Context>,
    state: Arc<AtomicBool>,
    workers: u32,
}

impl Drop for Pusher {
    fn drop(&mut self) {
        self.state.store(false, Ordering::Release);
    }
}

impl Pusher {
    pub fn new(ctx: Arc<Context>, mut workers: u32) -> Self {
        if workers == 0 {
            workers = 1
        }

        Pusher {
            ctx,
            workers,
            state: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> Result<()> {
        let ctx = self.ctx.clone();
        let state = self.state.clone();
        if let Err(_) = state.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) {
            return Err(anyhow!("already running"));
        }

        let workers = self.workers;
        task::spawn(async move {
            let mut poller = Poller::new(ctx, workers);
            poller.start_polling(state.as_ref()).await;
        });

        Ok(())
    }
}
