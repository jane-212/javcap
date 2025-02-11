use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use http_client::Client;
use log::info;
use nfo::Nfo;
use scraper::{Html, Selector};
use video::VideoType;

use super::{select, Finder};

select!(
    home_item: "body > div.subtitles > div > div > div > table > tbody > tr > td:nth-child(1) > a"
    detail_download_url: "#download_zh-CN"
);

pub struct SubtitleCat {
    client: Client,
    selectors: Selectors,
}

impl SubtitleCat {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<SubtitleCat> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let selectors = Selectors::new().with_context(|| "build selectors")?;

        let subtitle_cat = SubtitleCat { client, selectors };
        Ok(subtitle_cat)
    }
}

impl Display for SubtitleCat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "subtitle cat")
    }
}

#[async_trait]
impl Finder for SubtitleCat {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => true,
            VideoType::Fc2(_) => true,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder().id(key).build();

        let url = self.find_detail(key).await.with_context(|| "find detail")?;
        let subtitle = self
            .find_subtitle_in_detail(&url)
            .await
            .with_context(|| format!("find subtitle in detail {url}"))?;
        let subtitle = self
            .client
            .wait()
            .await
            .get(subtitle)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        if subtitle.contains("html") && subtitle.contains("404") {
            bail!("subtitle downloaded, but found 404 html in srt file");
        }
        nfo.set_subtitle(subtitle.into_bytes());

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}

impl SubtitleCat {
    async fn find_subtitle_in_detail(&self, url: &str) -> Result<String> {
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);
        html.select(&self.selectors.detail_download_url)
            .next()
            .and_then(|node| {
                node.attr("href")
                    .map(|href| format!("https://www.subtitlecat.com{href}"))
            })
            .ok_or_else(|| anyhow!("download url not found"))
    }

    async fn find_detail(&self, key: &VideoType) -> Result<String> {
        let url = "https://www.subtitlecat.com/index.php";
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("search", key.to_string())])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);
        let possible_names = match &key {
            VideoType::Jav(id, number) => {
                vec![format!("{id}-{number}"), format!("{id}{number}")]
            }
            VideoType::Fc2(number) => vec![
                format!("FC2-{number}"),
                format!("FC2-PPV-{number}"),
                format!("FC2PPV-{number}"),
                format!("FC2PPV{number}"),
            ],
        };

        html.select(&self.selectors.home_item)
            .find(|item| {
                let title = item.text().collect::<String>();
                possible_names.iter().any(|name| title.contains(name))
            })
            .and_then(|node| {
                node.attr("href")
                    .map(|href| format!("https://www.subtitlecat.com/{href}"))
            })
            .ok_or_else(|| anyhow!("subtitle not found"))
    }
}
