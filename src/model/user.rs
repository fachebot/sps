use anyhow::Result;
use sqlx::{Pool, Postgres};

#[derive(sqlx::FromRow)]
#[sqlx(type_name = "user")]
pub struct User {
    pub id: i64,
    pub wallet_address: String,
    pub creation_time: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(wallet_address: &str) -> Self {
        User {
            id: 0,
            wallet_address: String::from(wallet_address),
            creation_time: chrono::Utc::now(),
        }
    }
}

pub struct UserModel {
    pool: Pool<Postgres>,
}

impl UserModel {
    pub fn new(pool: &Pool<Postgres>) -> Self {
        UserModel { pool: pool.clone() }
    }

    pub async fn insert(&self, data: &User) -> Result<i64> {
        let query = r#"INSERT INTO "user"("wallet_address", "creation_time") VALUES($1, $2) RETURNING "id""#;
        let row: (i64,) = sqlx::query_as(query)
            .bind(&data.wallet_address)
            .bind(&data.creation_time)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_one_by_wallet_address(&self, wallet_address: &str) -> Result<User> {
        let query = r#"SELECT * FROM "user" WHERE "wallet_address" = $1"#;
        let user = sqlx::query_as(query)
            .bind(wallet_address)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }
}
