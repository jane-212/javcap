use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use video::VideoType;

use super::Finder;

pub struct Airav {
    client: Client,
}

impl Airav {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Airav> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;

        let airav = Airav { client };
        Ok(airav)
    }
}

impl Display for Airav {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "airav")
    }
}

#[async_trait]
impl Finder for Airav {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => true,
            VideoType::Fc2(_) => true,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let nfo = Nfo::builder()
            .id(key)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
