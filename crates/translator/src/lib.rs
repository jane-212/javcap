mod deepl;
mod openai;
mod youdao;

use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use config::Config;
use config::Translator as CfgTranslator;
use deepl::DeepL;
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
                            .build()
                            .with_context(|| "build youdao client")?;
                        let limiter = Ratelimiter::builder(1, Duration::from_secs(1))
                            .initial_available(1)
                            .build()
                            .with_context(|| "build limiter")?;

                        (limiter, Arc::new(handler) as Arc<dyn Handler>)
                    }
                    CfgTranslator::DeepSeek { base, model, key } => {
                        let handler = Openai::builder()
                            .base(base)
                            .model(model)
                            .key(key)
                            .timeout(timeout)
                            .maybe_proxy(proxy.clone())
                            .build()
                            .with_context(|| "build deepseek client")?;
                        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
                            .initial_available(1)
                            .build()
                            .with_context(|| "build limiter")?;

                        (limiter, Arc::new(handler) as Arc<dyn Handler>)
                    }
                    CfgTranslator::Openai { base, model, key } => {
                        let handler = Openai::builder()
                            .base(base)
                            .model(model)
                            .key(key)
                            .timeout(timeout)
                            .maybe_proxy(proxy.clone())
                            .build()
                            .with_context(|| "build openai client")?;
                        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
                            .initial_available(1)
                            .build()
                            .with_context(|| "build limiter")?;

                        (limiter, Arc::new(handler) as Arc<dyn Handler>)
                    }
                    CfgTranslator::DeepL => {
                        let handler = DeepL::builder()
                            .maybe_proxy(proxy.clone())
                            .build()
                            .with_context(|| "build deepl client")?;
                        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
                            .initial_available(1)
                            .build()
                            .with_context(|| "build limiter")?;

                        (limiter, Arc::new(handler) as Arc<dyn Handler>)
                    }
                };
                handlers.push(handler);
            }
        }
        let translator = Translator { handlers };
        if translator.handlers.is_empty() {
            info!("translate disabled");
        }

        Ok(translator)
    }

    async fn wait(&self) -> Option<Arc<dyn Handler>> {
        let mut times = Vec::with_capacity(self.handlers.len());

        'outer: loop {
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

            times.clear();
        }
    }

    pub async fn translate(&self, content: &str) -> Result<Option<String>> {
        let Some(handler) = self.wait().await else {
            return Ok(None);
        };
        let translated = handler
            .translate(content)
            .await
            .with_context(|| format!("in translator {handler}"))?;

        Ok(Some(translated))
    }
}

#[async_trait]
trait Handler: Send + Sync + Display {
    async fn translate(&self, content: &str) -> Result<String>;
}
