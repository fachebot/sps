use anyhow::Result;
use sqlx::{Pool, Postgres};

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
    pub transport: i64,
    pub state: String,
    pub retry_count: i32,
    pub reason: Option<String>,
    pub creation_time: chrono::DateTime<chrono::Utc>,
}

impl Task {
    pub fn new(message_id: i64, user_id: i64, transport: i64) -> Self {
        Task {
            id: 0,
            message_id,
            user_id,
            transport,
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
        let query = r#"INSERT INTO "task"("message_id", "user_id", "transport", "state", "retry_count", "reason", "creation_time") VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING "id""#;
        let row: (i64,) = sqlx::query_as(query)
            .bind(data.message_id)
            .bind(data.user_id)
            .bind(data.transport)
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
}
