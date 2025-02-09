use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use http_client::Client;
use log::info;
use nfo::Nfo;
use video::VideoType;

use super::Finder;

pub struct Missav {
    client: Client,
}

impl Missav {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Missav> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(2)
            .maybe_proxy(proxy)
            .build()?;

        let missav = Missav { client };
        Ok(missav)
    }

    async fn get_fanart(&self, name: &str) -> Result<Vec<u8>> {
        let url = format!("https://fourhoi.com/{}/cover-n.jpg", name.to_lowercase());
        let img = self
            .client
            .wait()
            .await
            .get(url)
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();

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

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
