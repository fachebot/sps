use crate::config::Config;
use crate::model;
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub struct Context {
    pub pool: Pool<Postgres>,
    pub message_model: model::MessageModel,
    pub task_model: model::TaskModel,
    pub token_model: model::TokenModel,
    pub transport_model: model::TransportModel,
    pub user_model: model::UserModel,
}

impl Context {
    pub async fn new(c: &Config) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&c.postgres.dsn)
            .await?;

        let ctx = Context {
            pool: pool.clone(),
            message_model: model::MessageModel::new(&pool),
            task_model: model::TaskModel::new(&pool),
            token_model: model::TokenModel::new(&pool),
            transport_model: model::TransportModel::new(&pool),
            user_model: model::UserModel::new(&pool),
        };

        Ok(ctx)
    }
}
