use async_std::fs::File;
use async_std::io::ReadExt;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Server {
    pub port: u16,
    pub access_expire: i64,
    pub access_secret: String,
}

#[derive(Clone, Deserialize)]
pub struct Postgres {
    pub dsn: String,
}

#[derive(Clone, Deserialize)]
pub struct Telegram {
    pub url: String,
    pub token: String,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub server: Server,
    pub postgres: Postgres,
    pub telegram: Telegram,
}

pub async fn must_load(filename: &str) -> Config {
    let mut file = File::open(filename).await.unwrap();

    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf).await.unwrap();

    toml::from_slice(&buf).unwrap()
}
