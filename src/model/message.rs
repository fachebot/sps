use anyhow::Result;
use sqlx::{Pool, Postgres};

#[derive(sqlx::FromRow)]
#[sqlx(type_name = "message")]
pub struct Message {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: String,
    pub creation_time: chrono::DateTime<chrono::Utc>,
}

impl Message {
    pub fn new(user_id: i64, title: &str, content: &str) -> Self {
        Message {
            id: 0,
            user_id,
            title: String::from(title),
            content: String::from(content),
            creation_time: chrono::Utc::now(),
        }
    }
}

pub struct MessageModel {
    pool: Pool<Postgres>,
}

impl MessageModel {
    pub fn new(pool: Pool<Postgres>) -> Self {
        MessageModel { pool }
    }

    pub async fn insert(&self, data: &Message) -> Result<i64> {
        let query = r#"INSERT INTO "message"("user_id", "title", "content", "creation_time") VALUES($1, $2, $3, $4) RETURNING "id""#;
        let row: (i64,) = sqlx::query_as(query)
            .bind(data.user_id)
            .bind(&data.title)
            .bind(&data.content)
            .bind(&data.creation_time)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_one_by_id(&self, id: i64) -> Result<Message> {
        let query = r#"SELECT * FROM "message" WHERE "id" = $1"#;
        let message = sqlx::query_as(query)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(message)
    }

    pub async fn find_all_by_user_id(&self, user_id: i64) -> Result<Vec<Message>> {
        let query = r#"SELECT * FROM "message" WHERE "user_id" = $1 ORDER BY "id" DESC"#;
        let messages = sqlx::query_as(query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(messages)
    }
}
