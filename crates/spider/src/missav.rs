use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use video::VideoType;

use super::Finder;

pub struct Missav {
    client: Client,
}

impl Missav {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Missav> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;

        let missav = Missav { client };
        Ok(missav)
    }

    async fn get_fanart(&self, key: &VideoType) -> Result<Vec<u8>> {
        let url = format!(
            "https://fourhoi.com/{}/cover-n.jpg",
            key.to_string().to_lowercase()
        );
        let img = self
            .client
            .wait()
            .await
            .get(&url)
            .send()
            .await
            .with_context(|| format!("send to {url}"))?
            .error_for_status()
            .with_context(|| "error status")?
            .bytes()
            .await
            .with_context(|| format!("decode to bytes from {url}"))?
            .to_vec();

        Ok(img)
    }
}

impl Display for Missav {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missav")
    }
}

#[async_trait]
impl Finder for Missav {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => true,
            VideoType::Fc2(_) => true,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder()
            .id(key)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        let fanart = self.get_fanart(key).await.with_context(|| "get fanart")?;
        nfo.set_fanart(fanart);

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
