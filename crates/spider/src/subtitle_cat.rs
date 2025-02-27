use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::Nfo;
use scraper::Html;
use video::VideoType;

use super::{Finder, select};

const HOST: &str = "https://www.subtitlecat.com";

select!(
    home_item: "body > div.subtitles > div > div > div > table > tbody > tr > td:nth-child(1) > a"
    detail_download_url: "#download_zh-CN"
);

pub struct SubtitleCat {
    base_url: String,
    client: Client,
    selectors: Selectors,
}

#[bon]
impl SubtitleCat {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<SubtitleCat> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let selectors = Selectors::new().with_context(|| "build selectors")?;
        let base_url = match base_url {
            Some(url) => url,
            None => String::from(HOST),
        };

        let subtitle_cat = SubtitleCat {
            base_url,
            client,
            selectors,
        };
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
            VideoType::Other(_) => false,
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

        info!("{nfo:?}");
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
                    .map(|href| format!("{}{href}", self.base_url))
            })
            .ok_or_else(|| anyhow!("download url not found"))
    }

    async fn find_detail(&self, key: &VideoType) -> Result<String> {
        let url = format!("{}/index.php", self.base_url);
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
            VideoType::Other(title) => vec![title.clone()],
        };

        html.select(&self.selectors.home_item)
            .find(|item| {
                let title = item.text().collect::<String>();
                possible_names.iter().any(|name| title.contains(name))
            })
            .and_then(|node| {
                node.attr("href")
                    .map(|href| format!("{}/{href}", self.base_url))
            })
            .ok_or_else(|| anyhow!("subtitle not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn finder() -> Result<SubtitleCat> {
        SubtitleCat::builder()
            .timeout(Duration::from_secs(10))
            .build()
    }

    #[test]
    fn test_support() -> Result<()> {
        let finder = finder()?;
        let videos = [
            (VideoType::Jav("STARS".to_string(), "804".to_string()), true),
            (VideoType::Fc2("3061625".to_string()), true),
        ];
        for (video, supported) in videos {
            assert_eq!(finder.support(&video), supported);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find() -> Result<()> {
        let finder = finder()?;
        let cases = [
            (VideoType::Jav("IPX".to_string(), "443".to_string()), {
                Nfo::builder().id("IPX-443").build()
            }),
            (VideoType::Jav("ROYD".to_string(), "108".to_string()), {
                Nfo::builder().id("ROYD-108").build()
            }),
            (VideoType::Jav("STARS".to_string(), "804".to_string()), {
                Nfo::builder().id("STARS-804").build()
            }),
            (VideoType::Fc2("3061625".to_string()), {
                Nfo::builder().id("FC2-PPV-3061625").build()
            }),
        ];
        for (video, expected) in cases {
            let actual = finder.find(&video).await?;
            assert!(actual.fanart().is_empty());
            assert!(actual.poster().is_empty());
            assert!(!actual.subtitle().is_empty());
            assert_eq!(actual, expected);
        }

        Ok(())
    }
}
