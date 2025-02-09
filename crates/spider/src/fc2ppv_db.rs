use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use http_client::Client;
use log::{info, warn};
use nfo::Nfo;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use video::VideoType;

use super::Finder;

pub struct Fc2ppvDB {
    client: Client,
}

impl Fc2ppvDB {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Fc2ppvDB> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(2)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;

        let fc2ppv_db = Fc2ppvDB { client };
        Ok(fc2ppv_db)
    }
}

#[async_trait]
impl Finder for Fc2ppvDB {
    fn name(&self) -> &'static str {
        "fc2ppv db"
    }

    async fn find(&self, key: VideoType) -> Result<Nfo> {
        let name = key.name();
        let mut nfo = Nfo::new(&name);

        match key {
            VideoType::Jav(_, _) => {
                warn!("jav type video not supported, skip({name})");
                return Ok(nfo);
            }
            VideoType::Fc2(_) => {}
        }

        nfo.set_country("日本".to_string());
        nfo.set_mpaa("NC-17".to_string());

        let url = "https://fc2ppvdb.com/search";
        let name = match key {
            VideoType::Jav(id, key) => format!("{id}-{key}"),
            VideoType::Fc2(key) => key,
        };
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("stype", "title"), ("keyword", &name)])
            .send()
            .await
            .with_context(|| format!("send to {url}"))?
            .text()
            .await
            .with_context(|| format!("decode to text from {url}"))?;
        {
            let html = Document::from(text.as_str());
            let Some(container) = html.find(Name("div").and(Class("container"))).next() else {
                bail!("container not found when find {name}");
            };
        }

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
