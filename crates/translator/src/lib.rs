mod youdao;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use config::Config;
use config::Translator as CfgTranslator;
use tokio::time;
use youdao::Youdao;

pub struct Translator {
    handlers: Vec<Arc<dyn Handler>>,
}

impl Translator {
    pub fn new(config: &Config) -> Result<Translator> {
        let timeout = Duration::from_secs(config.network.timeout);
        let proxy = &config.network.proxy;
        let mut handlers = vec![];
        if let Some(translators) = &config.translators {
            for translator in translators {
                let handler = match translator {
                    CfgTranslator::Youdao { key, secret } => {
                        Youdao::new(key, secret, timeout, proxy.clone())?
                    }
                };
                handlers.push(Arc::new(handler) as Arc<dyn Handler>);
            }
        }
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
