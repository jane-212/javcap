use async_trait::async_trait;

pub mod avatar;
pub mod video;

#[async_trait]
pub trait Task: Send + Sync {
    async fn run(&mut self) -> anyhow::Result<()>;
}
