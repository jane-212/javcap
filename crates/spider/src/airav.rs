use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use scraper::{Html, Selector};
use video::VideoType;

use super::{select, Finder};

select!(
    home_title: "body > div:nth-child(4) > div > div.row.row-cols-2.row-cols-lg-4.g-2.mt-0 > div:nth-child(1) > div > div.oneVideo-body > h5"
    home_fanart: "body > div:nth-child(4) > div > div.row.row-cols-2.row-cols-lg-4.g-2.mt-0 > div:nth-child(1) > div > div.oneVideo-top > a > img"
    home_url: "body > div:nth-child(4) > div > div.row.row-cols-2.row-cols-lg-4.g-2.mt-0 > div:nth-child(1) > div > div.oneVideo-top > a"
    detail_date: "body > div:nth-child(4) > div.container > div > div.col-lg-9.col-12.pt-3 > div.video-item > div.me-4"
    detail_plot: "body > div:nth-child(4) > div.container > div > div.col-lg-9.col-12.pt-3 > div.video-info > p"
    detail_name: "body > div:nth-child(4) > div.container > div > div.col-lg-9.col-12.pt-3 > div.video-info > div > ul > li"
    detail_tag: "a"
);

pub struct Airav {
    client: Client,
    selectors: Selectors,
}

impl Airav {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Airav> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let selectors = Selectors::new().with_context(|| "build selectors")?;

        let airav = Airav { client, selectors };
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
        let mut nfo = Nfo::builder()
            .id(key)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        let (url, fanart) = self
            .find_in_home(key, &mut nfo)
            .await
            .with_context(|| "find in home")?;
        if let Some(fanart) = fanart {
            let fanart = self
                .client
                .wait()
                .await
                .get(fanart)
                .send()
                .await?
                .bytes()
                .await?
                .to_vec();
            nfo.set_fanart(fanart);
        }
        if let Some(url) = url {
            self.find_detail(&url, &mut nfo)
                .await
                .with_context(|| "find detail")?;
        }

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}

impl Airav {
    async fn find_in_home(
        &self,
        key: &VideoType,
        nfo: &mut Nfo,
    ) -> Result<(Option<String>, Option<String>)> {
        let url = "https://airav.io/search_result";
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("kw", key.to_string())])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        if let Some(title) = html
            .select(&self.selectors.home_title)
            .next()
            .map(|node| node.text().collect())
        {
            nfo.set_title(title);
        }

        let fanart = html
            .select(&self.selectors.home_fanart)
            .next()
            .and_then(|node| node.attr("src").map(String::from));

        let url = html
            .select(&self.selectors.home_url)
            .next()
            .and_then(|node| {
                node.attr("href")
                    .map(|href| format!("https://airav.io{href}"))
            });

        Ok((url, fanart))
    }

    async fn find_detail(&self, url: &str, nfo: &mut Nfo) -> Result<()> {
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

        if let Some(date) = html
            .select(&self.selectors.detail_date)
            .next()
            .and_then(|node| node.text().last())
            .and_then(|text| text.split_once(' ').map(|(date, _)| date))
            .map(String::from)
        {
            nfo.set_premiered(date);
        }

        if let Some(plot) = html
            .select(&self.selectors.detail_plot)
            .next()
            .map(|node| node.text().collect())
        {
            nfo.set_plot(plot);
        }

        for item in html.select(&self.selectors.detail_name) {
            let Some(name) = item.text().next() else {
                continue;
            };

            match name.trim().trim_end_matches('：') {
                "女優" => {
                    for tag in item.select(&self.selectors.detail_tag) {
                        let tag = tag.text().collect();
                        nfo.actors_mut().insert(tag);
                    }
                }
                "標籤" => {
                    for tag in item.select(&self.selectors.detail_tag) {
                        let tag = tag.text().collect();
                        nfo.genres_mut().insert(tag);
                    }
                }
                "廠商" => {
                    if let Some(tag) = item
                        .select(&self.selectors.detail_tag)
                        .next()
                        .map(|node| node.text().collect())
                    {
                        nfo.set_studio(tag);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
