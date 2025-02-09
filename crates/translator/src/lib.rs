mod openai;
mod youdao;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use config::Config;
use config::Translator as CfgTranslator;
use log::info;
use openai::Openai;
use ratelimit::Ratelimiter;
use tokio::time;
use youdao::Youdao;

pub struct Translator {
    handlers: Vec<(Ratelimiter, Arc<dyn Handler>)>,
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
                        let handler = Youdao::builder()
                            .key(key)
                            .secret(secret)
                            .timeout(timeout)
                            .maybe_proxy(proxy.clone())
                            .build()?;
                        let limiter = Ratelimiter::builder(1, Duration::from_secs(1))
                            .initial_available(1)
                            .build()?;

                        (limiter, Arc::new(handler) as Arc<dyn Handler>)
                    }
                    CfgTranslator::DeepSeek { base, model, key } => {
                        let handler = Openai::builder()
                            .base(base)
                            .model(model)
                            .key(key)
                            .timeout(timeout)
                            .maybe_proxy(proxy.clone())
                            .build()?;
                        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
                            .initial_available(1)
                            .build()?;

                        (limiter, Arc::new(handler) as Arc<dyn Handler>)
                    }
                    CfgTranslator::Openai { base, model, key } => {
                        let handler = Openai::builder()
                            .base(base)
                            .model(model)
                            .key(key)
                            .timeout(timeout)
                            .maybe_proxy(proxy.clone())
                            .build()?;
                        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
                            .initial_available(1)
                            .build()?;

                        (limiter, Arc::new(handler) as Arc<dyn Handler>)
                    }
                };
                handlers.push(handler);
            }
        }
        let translator = Translator { handlers };
        if translator.handlers.is_empty() {
            info!("未启用翻译");
        }

        Ok(translator)
    }

    async fn wait(&self) -> Option<Arc<dyn Handler>> {
        let handler = 'outer: loop {
            let mut times = Vec::with_capacity(self.handlers.len());
            for handler in self.handlers.iter() {
                match handler.0.try_wait() {
                    Ok(_) => break 'outer Some(handler.1.clone()),
                    Err(time) => times.push(time),
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
}
