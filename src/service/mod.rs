use crate::config::Config;
use crate::model;
use anyhow::Result;
use async_std::sync::{Arc, Mutex};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub struct Context {
    pub conf: Config,
    pub pool: Pool<Postgres>,
    pub redis_client: redis::Client,
    pub redis_connection: Mutex<redis::aio::Connection>,
    pub message_model: model::MessageModel,
    pub task_model: model::TaskModel,
    pub transport_model: model::TransportModel,
    pub user_model: model::UserModel,
}

impl Context {
    pub async fn make_pointer(c: &Config) -> Result<Arc<Self>> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&c.postgres.dsn)
            .await?;

        let redis_client = redis::Client::open(c.redis.url.as_str())?;
        let redis_connection = redis_client.get_async_std_connection().await?;

        let ctx = Context {
            conf: c.clone(),
            pool: pool.clone(),
            redis_client,
            redis_connection: Mutex::new(redis_connection),
            message_model: model::MessageModel::new(pool.clone()),
            task_model: model::TaskModel::new(pool.clone()),
            transport_model: model::TransportModel::new(pool.clone()),
            user_model: model::UserModel::new(pool.clone()),
        };

        Ok(Arc::new(ctx))
    }
}
