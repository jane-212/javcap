use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{bail, Context, Result};
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
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;

        let subtitle_cat = SubtitleCat { client };
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

        let url = "https://www.subtitlecat.com/index.php";
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("search", key.to_string())])
            .send()
            .await
            .with_context(|| format!("send to {url}"))?
            .text()
            .await
            .with_context(|| format!("decode to text from {url}"))?;
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
            .get(&url)
            .send()
            .await
            .with_context(|| format!("send to {url}"))?
            .text()
            .await
            .with_context(|| format!("decode to text from {url}"))?;
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
                .get(&url)
                .send()
                .await
                .with_context(|| format!("send to {url}"))?
                .error_for_status()
                .with_context(|| "error status")?
                .text()
                .await
                .with_context(|| format!("decode to text from {url}"))?;
            if subtitle.contains("html") && subtitle.contains("404") {
                bail!("downloaded subtitle for {key}, but found 404 html in srt file");
            }
            nfo.set_subtitle(subtitle.into_bytes());
        }

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
