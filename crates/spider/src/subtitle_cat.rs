use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use log::info;
use nfo::Nfo;
use ratelimit::Ratelimiter;
use reqwest::{Client, Proxy};
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use tokio::time;
use video::VideoType;

use super::Finder;

pub struct SubtitleCat {
    limiter: Ratelimiter,
    client: Client,
}

impl SubtitleCat {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<SubtitleCat> {
        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
            .initial_available(1)
            .build()?;
        let mut client_builder = Client::builder().timeout(timeout);
        if let Some(url) = proxy {
            let proxy = Proxy::https(url)?;
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder.build()?;
        let subtitle_cat = SubtitleCat { client, limiter };

        Ok(subtitle_cat)
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
impl Finder for SubtitleCat {
    async fn find(&self, key: VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::new(key.name());

        let url = "https://www.subtitlecat.com/index.php";
        self.wait_limiter().await;
        let text = self
            .client
            .get(url)
            .query(&[("search", key.name())])
            .send()
            .await?
            .text()
            .await?;
        let url = {
            let html = Document::from(text.as_str());
            let mut found = None;
            for item in html.find(Name("table").and(Class("table")).descendant(Name("tr"))) {
                let Some(a) = item.find(Name("a")).next() else {
                    continue;
                };

                let Some(url) = a
                    .attr("href")
                    .map(|href| format!("https://www.subtitlecat.com/{href}"))
                else {
                    continue;
                };

                let possible_names = match &key {
                    VideoType::Jav(id, key) => vec![format!("{id}-{key}"), format!("{id}{key}")],
                    VideoType::Fc2(key) => vec![
                        format!("FC2-{key}"),
                        format!("FC2-PPV-{key}"),
                        format!("FC2PPV-{key}"),
                        format!("FC2PPV{key}"),
                    ],
                };

                let title = item.text();
                if possible_names.iter().any(|name| title.contains(name)) {
                    found = Some(url);
                    break;
                }
            }

            found
        };
        let Some(url) = url else {
            return Ok(nfo);
        };

        self.wait_limiter().await;
        let text = self.client.get(url).send().await?.text().await?;
        let url = {
            let html = Document::from(text.as_str());

            let mut url = None;
            for item in html.find(
                Name("div")
                    .and(Class("container"))
                    .descendant(Name("div").and(Class("sub-single"))),
            ) {
                if let Some(download_url) = item
                    .find(Name("a").and(Attr("id", "download_zh-CN")))
                    .next()
                    .and_then(|node| {
                        node.attr("href")
                            .map(|href| format!("https://www.subtitlecat.com{href}"))
                    })
                {
                    url = Some(download_url);
                    break;
                }
            }

            url
        };
        if let Some(url) = url {
            self.wait_limiter().await;
            let subtitle = self.client.get(url).send().await?.text().await?;
            if subtitle.contains("html") && subtitle.contains("404") {
                return Ok(nfo);
            }
            nfo.set_subtitle(subtitle.into_bytes());
        }

        info!("从subtitle找到字幕 > {}", nfo.subtitle().len());
        Ok(nfo)
    }
}
