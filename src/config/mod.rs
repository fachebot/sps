use async_std::fs::File;
use async_std::io::ReadExt;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Pg {
    pub dsn: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub postgres: Pg,
}

pub async fn must_load(filename: &str) -> Config {
    let mut file = File::open(filename).await.unwrap();

    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf).await.unwrap();

    toml::from_slice(&buf).unwrap()
}
