use crate::model::Task;
use crate::service::Context;
use anyhow::{anyhow, Result};
use async_std::{channel, channel::Receiver, channel::Sender, sync::Arc, task};
use redis::aio::Connection;
use redis::{AsyncCommands, RedisResult};
use std::ops::{DerefMut, Index};
use std::sync::atomic::{AtomicBool, Ordering};

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
            let task = self.ctx.task_model.find_one_by_id(task_id).await;
            if let Err(err) = task {
                self.retry_task(&mut conn, task_id, err.to_string()).await;
                continue;
            }

            self.push(&mut conn, task.unwrap()).await;
        }
    }

    async fn push(&self, conn: &mut Connection, task: Task) {
        let message = self.ctx.message_model.find_one_by_id(task.message_id).await;
        if let Err(err) = message {
            self.retry_task(conn, task.id, err.to_string()).await;
            return;
        }

        let transport = self
            .ctx
            .transport_model
            .find_one_by_id(task.transport)
            .await;
        if let Err(err) = transport {
            self.retry_task(conn, task.id, err.to_string()).await;
            return;
        }

        let message = message.unwrap();
        let transport = transport.unwrap();
    }

    async fn retry_task(&self, conn: &mut Connection, task_id: i64, reason: String) {}
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
            senders: Vec::new(),
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
                log::error!("[Pusher] redis read error, {}", err.to_string());

                task::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            let task_ids = result.unwrap();
            if task_ids.is_empty() {
                task::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            let result: RedisResult<Vec<i64>> = guard
                .deref_mut()
                .zrembyscore(queue_name.as_str(), 0, ts)
                .await;
            if let Err(err) = result {
                log::error!("[Pusher] redis write error, {}", err.to_string());
            }

            for task_id in task_ids {
                let result = self.senders.index(self.next).send(task_id).await;
                if let Err(err) = result {
                    log::error!(
                        "[Pusher] failed to assign task, task_id: {}, {}",
                        task_id,
                        err.to_string()
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
