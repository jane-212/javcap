use super::info::Info;
use async_trait::async_trait;

mod app_world;
mod youdao;

pub use app_world::*;
pub use youdao::*;

#[async_trait]
pub trait Translator: Send + Sync {
    async fn translate<'a>(&mut self, info: &mut Info) -> anyhow::Result<()>;
}
