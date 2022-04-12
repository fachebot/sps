use crate::config::Config;
use crate::model;
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[derive(Clone)]
pub struct Context {
    pub pool: Pool<Postgres>,
    pub message_model: Arc<model::MessageModel>,
    pub task_model: Arc<model::TaskModel>,
    pub token_model: Arc<model::TokenModel>,
    pub transport_model: Arc<model::TransportModel>,
    pub user_model: Arc<model::UserModel>,
}

impl Context {
    pub async fn new(c: &Config) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&c.postgres.dsn)
            .await?;

        let ctx = Context {
            pool: pool.clone(),
            message_model: Arc::new(model::MessageModel::new(&pool)),
            task_model: Arc::new(model::TaskModel::new(&pool)),
            token_model: Arc::new(model::TokenModel::new(&pool)),
            transport_model: Arc::new(model::TransportModel::new(&pool)),
            user_model: Arc::new(model::UserModel::new(&pool)),
        };

        Ok(ctx)
    }
}
