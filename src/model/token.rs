use anyhow::Result;
use sqlx::{Pool, Postgres};

#[derive(sqlx::FromRow)]
#[sqlx(type_name = "task")]
pub struct Token {
    pub id: i64,
    pub user_id: i64,
    pub access_token: String,
    pub creation_time: chrono::DateTime<chrono::Utc>,
}

impl Token {
    pub fn new(user_id: i64, access_token: &str) -> Self {
        Token {
            id: 0,
            user_id,
            access_token: String::from(access_token),
            creation_time: chrono::Utc::now(),
        }
    }
}

pub struct TokenModel {
    pool: Pool<Postgres>,
}

impl TokenModel {
    pub fn new(pool: &Pool<Postgres>) -> Self {
        TokenModel { pool: pool.clone() }
    }

    pub async fn insert(&self, data: &Token) -> Result<i64> {
        let query = r#"INSERT INTO "token"("user_id", "access_token", "creation_time") VALUES($1, $2, $3) RETURNING "id""#;
        let row: (i64,) = sqlx::query_as(query)
            .bind(data.user_id)
            .bind(&data.access_token)
            .bind(&data.creation_time)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_one_by_user_id(&self, user_id: i64) -> Result<Token> {
        let query = r#"SELECT * FROM "token" WHERE "user_id" = $1"#;
        let token = sqlx::query_as(query)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(token)
    }

    pub async fn find_one_by_access_token(&self, access_token: &str) -> Result<Token> {
        let query = r#"SELECT * FROM "token" WHERE "access_token" = $1"#;
        let token = sqlx::query_as(query)
            .bind(access_token)
            .fetch_one(&self.pool)
            .await?;
        Ok(token)
    }
}
