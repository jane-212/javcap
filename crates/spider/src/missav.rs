use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use nfo::Nfo;
use ratelimit::Ratelimiter;
use reqwest::Client;
use tokio::sync::Mutex;
use tokio::time;

use super::Finder;

pub struct Missav {
    client: Client,
    limiter: Mutex<Ratelimiter>,
}

impl Missav {
    pub fn new() -> Result<Missav> {
        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
            .initial_available(1)
            .build()?;
        let limiter = Mutex::new(limiter);
        let client = Client::new();

        let missav = Missav { client, limiter };
        Ok(missav)
    }

    async fn wait_limiter(&self) {
        let limiter = self.limiter.lock().await;
        loop {
            match limiter.try_wait() {
                Ok(_) => break,
                Err(sleep) => time::sleep(sleep).await,
            }
        }
    }

    async fn get_fanart(&self, key: &str) -> Result<Vec<u8>> {
        let url = format!("https://fourhoi.com/{}/cover-n.jpg", key.to_lowercase());
        let img = self.client.get(url).send().await?.bytes().await?.to_vec();

        Ok(img)
    }
}

#[async_trait]
impl Finder for Missav {
    async fn find(&self, key: &str) -> Result<Nfo> {
        self.wait_limiter().await;

        let mut nfo = Nfo::new(key);
        nfo.set_country("日本".to_string());

        let fanart = self.get_fanart(key).await?;
        nfo.set_fanart(fanart);

        Ok(nfo)
    }
}
