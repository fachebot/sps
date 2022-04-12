use anyhow::Result;
use sqlx::{Pool, Postgres};

pub mod transport_type {
    pub const TELEGRAM: &str = "telegram";
}

#[derive(sqlx::FromRow)]
#[sqlx(type_name = "transport")]
pub struct Transport {
    pub id: i64,
    pub user_id: i64,
    #[sqlx(rename = "type")]
    pub transport_type: String,
    pub chat_id: Option<String>,
    pub connected: bool,
    pub creation_time: chrono::DateTime<chrono::Utc>,
}

impl Transport {
    pub fn new(user_id: i64, transport_type: &str) -> Self {
        Transport {
            id: 0,
            user_id,
            transport_type: String::from(transport_type),
            chat_id: None,
            connected: false,
            creation_time: chrono::Utc::now(),
        }
    }
}

pub struct TransportModel {
    pool: Pool<Postgres>,
}

impl TransportModel {
    pub fn new(pool: &Pool<Postgres>) -> Self {
        TransportModel { pool: pool.clone() }
    }

    pub async fn insert(&self, data: &Transport) -> Result<i64> {
        let query = r#"INSERT INTO "transport"("user_id", "type", "chat_id", "connected", "creation_time") VALUES($1, $2, $3, $4, $5) RETURNING "id""#;
        let row: (i64,) = sqlx::query_as(query)
            .bind(data.user_id)
            .bind(&data.transport_type)
            .bind(&data.chat_id)
            .bind(&data.connected)
            .bind(&data.creation_time)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_all_by_user_id(&self, user_id: i64) -> Result<Vec<Transport>> {
        let query = r#"SELECT * FROM "transport" WHERE "user_id" = $1"#;
        let transports = sqlx::query_as(query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(transports)
    }

    pub async fn find_one_by_user_id_type(
        &self,
        user_id: i64,
        transport_type: &str,
    ) -> Result<Transport> {
        let query = r#"SELECT * FROM "transport" WHERE "user_id" = $1 AND "type" = $2"#;
        let transport = sqlx::query_as(query)
            .bind(user_id)
            .bind(transport_type)
            .fetch_one(&self.pool)
            .await?;
        Ok(transport)
    }
}
