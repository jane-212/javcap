use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use log::info;
use nfo::Nfo;
use ratelimit::Ratelimiter;
use reqwest::{Client, Proxy};
use tokio::time;
use video::VideoType;

use super::Finder;

pub struct Missav {
    client: Client,
    limiter: Ratelimiter,
}

impl Missav {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Missav> {
        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
            .initial_available(1)
            .build()?;
        let mut client_builder = Client::builder().timeout(timeout);
        if let Some(url) = proxy {
            let proxy = Proxy::https(url)?;
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder.build()?;

        let missav = Missav { client, limiter };
        Ok(missav)
    }

    async fn wait_limiter(&self) {
        loop {
            match self.limiter.try_wait() {
                Ok(_) => break,
                Err(sleep) => time::sleep(sleep).await,
            }
        }
    }

    async fn get_fanart(&self, name: &str) -> Result<Vec<u8>> {
        let url = format!("https://fourhoi.com/{}/cover-n.jpg", name.to_lowercase());
        self.wait_limiter().await;
        let img = self.client.get(url).send().await?.bytes().await?.to_vec();

        Ok(img)
    }
}

#[async_trait]
impl Finder for Missav {
    async fn find(&self, key: VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::new(key.name());
        nfo.set_country("日本".to_string());
        nfo.set_mpaa("NC-17".to_string());

        let fanart = self.get_fanart(&key.name()).await?;
        nfo.set_fanart(fanart);

        info!("从missav找到fanart > {}", nfo.fanart().len());
        Ok(nfo)
    }
}
