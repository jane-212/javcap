use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use scraper::{Html, Selector};
use video::VideoType;

use super::{select, Finder};

select!(
    home_item: "body > section > div > div.movie-list.h.cols-4.vcols-8 > div"
    home_item_id: "a > div.video-title > strong"
    home_title: "a"
    home_date: "a > div.meta"
    home_rating: "a > div.score > span"
    detail_block: "body > section > div > div.video-detail > div.video-meta-panel > div > div:nth-child(2) > nav > div.panel-block"
    detail_name: "strong"
    detail_value: "span"
);

pub struct Javdb {
    base_url: String,
    client: Client,
    selectors: Selectors,
}

#[bon]
impl Javdb {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Javdb> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(2)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let selectors = Selectors::new().with_context(|| "build selectors")?;

        let javdb = Javdb {
            base_url: base_url.unwrap_or("https://javdb.com".to_string()),
            client,
            selectors,
        };
        Ok(javdb)
    }
}

impl Display for Javdb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "javdb")
    }
}

#[async_trait]
impl Finder for Javdb {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => true,
            VideoType::Fc2(_) => false,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder()
            .id(key)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        let url = self
            .find_in_home(key, &mut nfo)
            .await
            .with_context(|| "find in home")?;
        self.find_detail(&url, &mut nfo)
            .await
            .with_context(|| format!("find detail {url}"))?;

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}

impl Javdb {
    async fn find_in_home(&self, key: &VideoType, nfo: &mut Nfo) -> Result<String> {
        let url = format!("{}/search", self.base_url);
        let text = self
            .client
            .wait()
            .await
            .get(&url)
            .query(&[("q", key.to_string().as_str()), ("f", "all")])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        let name = key.to_string();
        let Some(item) = html.select(&self.selectors.home_item).find(|node| {
            node.select(&self.selectors.home_item_id)
                .next()
                .map(|node| node.text().collect::<String>() == name)
                .unwrap_or(false)
        }) else {
            bail!("item not found");
        };

        if let Some(date) = item
            .select(&self.selectors.home_date)
            .next()
            .map(|node| node.text().collect::<String>())
        {
            nfo.set_premiered(date.trim().to_string());
        }

        if let Some(rating) = item
            .select(&self.selectors.home_rating)
            .next()
            .and_then(|node| node.text().last().map(|text| text.trim()))
            .map(|text| {
                text.chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect::<String>()
                    .parse::<f64>()
                    .unwrap_or_default()
            })
            .map(|rating| rating * 2.0)
        {
            nfo.set_rating(rating);
        }

        if let Some(title) = item
            .select(&self.selectors.home_title)
            .next()
            .and_then(|node| node.attr("title"))
        {
            nfo.set_title(title.to_string());
        }

        item.select(&self.selectors.home_title)
            .next()
            .and_then(|node| {
                node.attr("href")
                    .map(|href| format!("{}{href}", self.base_url))
            })
            .ok_or_else(|| anyhow!("detail url not found"))
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
        for block in html.select(&self.selectors.detail_block) {
            let Some(name) = block
                .select(&self.selectors.detail_name)
                .next()
                .map(|node| node.text().collect::<String>())
            else {
                continue;
            };
            let Some(value) = block
                .select(&self.selectors.detail_value)
                .next()
                .map(|node| node.text().collect::<String>())
            else {
                continue;
            };

            let name = name.trim_end_matches(":").trim();
            let value = value.trim();

            match name {
                "時長" => {
                    let runtime: u32 = value
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse()
                        .unwrap_or_default();
                    nfo.set_runtime(runtime);
                }
                "導演" => {
                    nfo.set_director(value.to_string());
                }
                "片商" => {
                    nfo.set_studio(value.to_string());
                }
                "類別" => {
                    let genres = value.split(",").collect::<Vec<_>>();
                    for genre in genres {
                        nfo.genres_mut().insert(genre.trim().to_string());
                    }
                }
                "演員" => {
                    let actors = value
                        .lines()
                        .map(|line| line.trim().trim_end_matches(['♂', '♀']))
                        .collect::<Vec<_>>();
                    for actor in actors {
                        nfo.actors_mut().insert(actor.to_string());
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
