use async_trait::async_trait;

pub mod network;

#[async_trait]
pub trait Checker: Send + Sync {
    async fn check(&self) -> anyhow::Result<()>;
}
