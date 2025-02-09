use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use http_client::Client;
use log::info;
use nfo::Nfo;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use video::VideoType;

use super::Finder;

pub struct SubtitleCat {
    client: Client,
}

impl SubtitleCat {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<SubtitleCat> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(2)
            .maybe_proxy(proxy)
            .build()?;

        let subtitle_cat = SubtitleCat { client };
        Ok(subtitle_cat)
    }
}

#[async_trait]
impl Finder for SubtitleCat {
    async fn find(&self, key: VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::new(key.name());

        let url = "https://www.subtitlecat.com/index.php";
        let text = self
            .client
            .wait()
            .await
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

        let text = self
            .client
            .wait()
            .await
            .get(url)
            .send()
            .await?
            .text()
            .await?;
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
            let subtitle = self
                .client
                .wait()
                .await
                .get(url)
                .send()
                .await?
                .text()
                .await?;
            if subtitle.contains("html") && subtitle.contains("404") {
                return Ok(nfo);
            }
            nfo.set_subtitle(subtitle.into_bytes());
        }

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
