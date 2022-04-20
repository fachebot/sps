use crate::model::task;
use anyhow::Result;
use sqlx::{Pool, Postgres};

pub async fn insert_message(
    pool: &Pool<Postgres>,
    user_id: i64,
    title: &str,
    content: &str,
    transports: &[i64],
) -> Result<Vec<i64>> {
    let mut tx = pool.begin().await?;
    let creation_time = chrono::Utc::now();

    let query = r#"INSERT INTO "message"("user_id", "title", "content", "creation_time") VALUES($1, $2, $3, $4) RETURNING "id""#;
    let row: (i64,) = sqlx::query_as(query)
        .bind(user_id)
        .bind(title)
        .bind(content)
        .bind(&creation_time)
        .fetch_one(&mut tx)
        .await?;

    let message_id = row.0;
    let retry_count = 0i32;
    let reason: Option<String> = None;

    let mut ids = Vec::<i64>::new();
    for transport in transports {
        let query = r#"INSERT INTO "task"("message_id", "user_id", "transport", "state", "retry_count", "reason", "creation_time") VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING "id""#;
        let row: (i64,) = sqlx::query_as(query)
            .bind(message_id)
            .bind(user_id)
            .bind(*transport)
            .bind(task::state::PENDING)
            .bind(retry_count)
            .bind(&reason)
            .bind(&creation_time)
            .fetch_one(&mut tx)
            .await?;

        ids.push(row.0);
    }

    tx.commit().await?;

    Ok(ids)
}
