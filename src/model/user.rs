use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{Pool, Postgres};
use std::str::FromStr;

#[derive(sqlx::FromRow)]
#[sqlx(type_name = "user")]
pub struct User {
    pub id: i64,
    pub open_id: uuid::Uuid,
    pub project_id: String,
    pub wallet_address: String,
    pub creation_time: chrono::DateTime<chrono::Utc>,
}

pub fn gen_project_id() -> String {
    let buf: Vec<u8> = thread_rng().sample_iter(&Alphanumeric).take(45).collect();
    String::from_utf8_lossy(buf.as_slice()).to_string()
}

impl User {
    pub fn new(wallet_address: &str) -> Self {
        User {
            id: 0,
            open_id: uuid::Uuid::new_v4(),
            project_id: gen_project_id(),
            wallet_address: String::from(wallet_address),
            creation_time: chrono::Utc::now(),
        }
    }
}

pub struct UserModel {
    pool: Pool<Postgres>,
}

impl UserModel {
    pub fn new(pool: Pool<Postgres>) -> Self {
        UserModel { pool }
    }

    pub async fn insert(&self, data: &User) -> Result<i64> {
        let query = r#"INSERT INTO "user"("open_id", "project_id", "wallet_address", "creation_time") VALUES($1, $2, $3, $4) RETURNING "id""#;
        let row: (i64,) = sqlx::query_as(query)
            .bind(data.open_id)
            .bind(&data.project_id)
            .bind(&data.wallet_address)
            .bind(&data.creation_time)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_one_by_open_id(&self, open_id: &str) -> Result<User> {
        let open_id = uuid::Uuid::from_str(open_id)?;
        let query = r#"SELECT * FROM "user" WHERE "open_id" = $1"#;
        let user = sqlx::query_as(query)
            .bind(open_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
    }

    pub async fn find_one_by_project_id(&self, project_id: &str) -> Result<User> {
        let query = r#"SELECT * FROM "user" WHERE "project_id" = $1"#;
        let user = sqlx::query_as(query)
            .bind(project_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(user)
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
