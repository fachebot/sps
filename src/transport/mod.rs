use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Transport {
    async fn push(&self, chat: &str, title: &str, message: &str) -> Result<()>;
}

pub mod telegram;
pub use telegram::*;
