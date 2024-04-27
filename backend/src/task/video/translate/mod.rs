use async_trait::async_trait;

use super::info::Info;

mod app_world;

pub use app_world::*;

#[async_trait]
pub trait Translator: Send + Sync {
    async fn translate<'a>(&mut self, info: &mut Info) -> anyhow::Result<()>;
}
