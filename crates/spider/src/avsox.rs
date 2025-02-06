use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use nfo::Nfo;
use ratelimit::Ratelimiter;
use reqwest::{Client, Proxy};
use tokio::time;
use video::VideoType;

use super::Finder;

pub struct Avsox {
    base_url: String,
    limiter: Ratelimiter,
    client: Client,
}

impl Avsox {
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Avsox> {
        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
            .initial_available(1)
            .build()?;
        let mut client_builder = Client::builder().timeout(timeout);
        if let Some(url) = proxy {
            let proxy = Proxy::https(url)?;
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder.build()?;
        let avsox = Avsox {
            base_url: base_url.unwrap_or("".to_string()),
            client,
            limiter,
        };

        Ok(avsox)
    }

    async fn wait_limiter(&self) {
        loop {
            match self.limiter.try_wait() {
                Ok(_) => break,
                Err(sleep) => time::sleep(sleep).await,
            }
        }
    }
}

#[async_trait]
impl Finder for Avsox {
    async fn find(&self, key: VideoType) -> Result<Nfo> {
        self.wait_limiter().await;

        let mut nfo = Nfo::new(key.name());
        nfo.set_country("日本".to_string());

        Ok(nfo)
    }
}
