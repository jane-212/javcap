use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use tokio::time;

pub struct Translator {
    handlers: Vec<Arc<dyn Handler>>,
}

impl Translator {
    pub fn new() -> Result<Translator> {
        let handlers = vec![];
        let translator = Translator { handlers };

        Ok(translator)
    }

    async fn wait(&self) -> Option<Arc<dyn Handler>> {
        let handler = 'outer: loop {
            let mut times = Vec::with_capacity(self.handlers.len());
            for handler in self.handlers.iter() {
                match handler.wait() {
                    Some(time) => times.push(time),
                    None => break 'outer Some(handler.clone()),
                }
            }

            times.sort();
            match times.first() {
                Some(sleep) => time::sleep(*sleep).await,
                None => break None,
            }
        };

        handler
    }

    pub async fn translate(&self, content: &str) -> Result<Option<String>> {
        let Some(handler) = self.wait().await else {
            return Ok(None);
        };
        let translated = handler.translate(content).await?;

        Ok(Some(translated))
    }
}

#[async_trait]
trait Handler: Send + Sync {
    async fn translate(&self, content: &str) -> Result<String>;
    fn wait(&self) -> Option<Duration>;
}
