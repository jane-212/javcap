use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use nfo::Nfo;
use video::VideoType;

use super::Finder;

const HOST: &str = "https://91porny.com";

pub struct Porny {
    base_url: String,
    client: Client,
}

#[bon]
impl Porny {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Porny> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let base_url = match base_url {
            Some(url) => url,
            None => String::from(HOST),
        };

        let porny = Porny { base_url, client };
        Ok(porny)
    }
}

impl Display for Porny {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "91 porny")
    }
}

#[async_trait]
impl Finder for Porny {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => false,
            VideoType::Fc2(_) => false,
            VideoType::Other(_) => true,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        todo!()
    }
}
