use super::info::Info;
use crate::select;
use reqwest::Client;
use scraper::{selectable::Selectable, Html};
use std::sync::Arc;

pub struct Subtitle {
    client: Arc<Client>,
}

impl Subtitle {
    const HOST: &'static str = "https://www.subtitlecat.com";

    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn find_subtitle(&self, info: &mut Info) -> anyhow::Result<()> {
        let Some(href) = self.find_item(info.get_id()).await? else {
            return Ok(());
        };
        let Some(subtitle_href) = self.load_subtitle(href).await? else {
            return Ok(());
        };

        let subtitle = self.client.get(subtitle_href).send().await?.text().await?;
        if subtitle.contains("html") && subtitle.contains("404") {
            return Ok(());
        }
        info.subtitle(subtitle.into_bytes());

        Ok(())
    }

    async fn load_subtitle(&self, href: impl AsRef<str>) -> anyhow::Result<Option<String>> {
        select!(
            item: "body > div.all-sub > div > div:nth-child(2) > div > div",
            language: "span:nth-child(2)",
            href: "span:nth-child(3) > a"
        );

        let res = self.client.get(href.as_ref()).send().await?.text().await?;
        let doc = Html::parse_document(&res);

        let Some(item) = doc.select(&selectors().item).find(|item| {
            item.select(&selectors().language)
                .next()
                .map(|lang| lang.inner_html().contains("Chinese"))
                .unwrap_or(false)
        }) else {
            return Ok(None);
        };

        let subtitle_href = item
            .select(&selectors().href)
            .next()
            .and_then(|href| href.attr("href"))
            .map(|href| format!("{}{}", Self::HOST, href));

        Ok(subtitle_href)
    }

    async fn find_item(&self, id: &str) -> anyhow::Result<Option<String>> {
        select!(
            item: "body > div.subtitles > div > div > div > table > tbody > tr > td:nth-child(1) > a"
        );

        let url = format!("https://www.subtitlecat.com/index.php?search={}", id);
        let res = self.client.get(url).send().await?.text().await?;
        let doc = Html::parse_document(&res);

        let Some(item) = doc
            .select(&selectors().item)
            .find(|item| item.inner_html().contains(id))
        else {
            return Ok(None);
        };

        Ok(item
            .attr("href")
            .map(|href| format!("{}/{}", Self::HOST, href)))
    }
}
