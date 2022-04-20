use anyhow::Result;
use sqlx::{Pool, Postgres};
use crate::model::Transport;

pub mod state {
    pub const PENDING: &str = "pending";
    pub const RETRYING: &str = "retrying";
    pub const FAIL: &str = "fail";
    pub const DONE: &str = "done";
}

#[derive(sqlx::FromRow)]
#[sqlx(type_name = "task")]
pub struct Task {
    pub id: i64,
    pub message_id: i64,
    pub user_id: i64,
    pub chat_id: String,
    pub transport: i64,
    pub transport_type: String,
    pub state: String,
    pub retry_count: i32,
    pub reason: Option<String>,
    pub creation_time: chrono::DateTime<chrono::Utc>,
}

impl Task {
    pub fn new(message_id: i64, user_id: i64, transport: &Transport) -> Self {
        let chat_id = transport.chat_id.as_ref().unwrap();

        Task {
            id: 0,
            message_id,
            user_id,
            chat_id: chat_id.clone(),
            transport: transport.id,
            transport_type: transport.transport_type.clone(),
            state: self::state::PENDING.into(),
            retry_count: 0,
            reason: None,
            creation_time: chrono::Utc::now(),
        }
    }
}

pub struct TaskModel {
    pool: Pool<Postgres>,
}

impl TaskModel {
    pub fn new(pool: Pool<Postgres>) -> Self {
        TaskModel { pool }
    }

    pub async fn insert(&self, data: &Task) -> Result<i64> {
        let query = r#"INSERT INTO "task"("message_id", "user_id", "chat_id", "transport", "transport_type", "state", "retry_count", "reason", "creation_time") VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING "id""#;
        let row: (i64,) = sqlx::query_as(query)
            .bind(data.message_id)
            .bind(data.user_id)
            .bind(&data.chat_id)
            .bind(data.transport)
            .bind(&data.transport_type)
            .bind(data.state.clone())
            .bind(data.retry_count)
            .bind(&data.reason)
            .bind(&data.creation_time)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_one_by_id(&self, id: i64) -> Result<Task> {
        let query = r#"SELECT * FROM "task" WHERE "id" = $1"#;
        let task = sqlx::query_as(query).bind(id).fetch_one(&self.pool).await?;
        Ok(task)
    }

    pub async fn set_done(&self, id: i64) -> Result<()> {
        let query = r#"UPDATE "task" SET "state" = $1 WHERE "id" = $2"#;
        sqlx::query(query)
            .bind(self::state::DONE)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
