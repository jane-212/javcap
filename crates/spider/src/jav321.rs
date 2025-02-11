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
    title: "body > div:nth-child(6) > div.col-md-7.col-md-offset-1.col-xs-12 > div:nth-child(1) > div.panel-heading > h3"
    plot: "body > div:nth-child(6) > div.col-md-7.col-md-offset-1.col-xs-12 > div:nth-child(1) > div.panel-body > div:nth-child(3) > div"
    poster: "body > div:nth-child(6) > div.col-md-7.col-md-offset-1.col-xs-12 > div:nth-child(1) > div.panel-body > div:nth-child(1) > div.col-md-3 > img"
    fanart: "body > div:nth-child(6) > div.col-md-3 > div:nth-child(1) > p > a > img"
    info: "body > div:nth-child(6) > div.col-md-7.col-md-offset-1.col-xs-12 > div:nth-child(1) > div.panel-body > div:nth-child(1) > div.col-md-9"
);

pub struct Jav321 {
    client: Client,
    selectors: Selectors,
}

impl Jav321 {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Jav321> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let selectors = Selectors::new().with_context(|| "build selectors")?;

        let jav321 = Jav321 { client, selectors };
        Ok(jav321)
    }
}

impl Display for Jav321 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "jav321")
    }
}

#[async_trait]
impl Finder for Jav321 {
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

        let (poster, fanart) = self
            .find_detail(key, &mut nfo)
            .await
            .with_context(|| "find detail")?;
        if let Some(poster) = poster {
            let poster = self
                .client
                .wait()
                .await
                .get(poster)
                .send()
                .await?
                .bytes()
                .await?;
            nfo.set_poster(poster.to_vec());
        }
        if let Some(fanart) = fanart {
            let fanart = self
                .client
                .wait()
                .await
                .get(fanart)
                .send()
                .await?
                .bytes()
                .await?;
            nfo.set_fanart(fanart.to_vec());
        }

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}

impl Jav321 {
    async fn find_detail(
        &self,
        key: &VideoType,
        nfo: &mut Nfo,
    ) -> Result<(Option<String>, Option<String>)> {
        let url = "https://www.jav321.com/search";
        let text = self
            .client
            .wait()
            .await
            .post(url)
            .form(&[("sn", key.to_string())])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        if let Some(title) = html
            .select(&self.selectors.title)
            .next()
            .and_then(|node| node.text().next().map(|text| text.trim()))
        {
            nfo.set_title(title.to_string());
        }

        if let Some(plot) = html
            .select(&self.selectors.plot)
            .next()
            .and_then(|node| node.text().next().map(|text| text.trim()))
        {
            nfo.set_plot(plot.to_string());
        }

        let poster = html
            .select(&self.selectors.poster)
            .next()
            .and_then(|node| node.attr("src").map(|src| src.to_string()));

        let fanart = html
            .select(&self.selectors.fanart)
            .next()
            .and_then(|node| node.attr("src").map(|src| src.to_string()));

        if let Some(info) = html.select(&self.selectors.info).next() {
            let mut s = Vec::new();
            for text in info.text() {
                if text.starts_with(":") {
                    s.push(":".to_string());
                    s.push(text.trim_start_matches(":").trim().to_string());
                } else {
                    s.push(text.trim().to_string());
                }
            }

            let mut v = Vec::new();
            while let Some(text) = s.pop() {
                if text.is_empty() {
                    continue;
                }

                if text != ":" {
                    v.push(text);
                    continue;
                }

                if let Some(name) = s.pop() {
                    match name.as_str() {
                        "メーカー" => {
                            if let Some(studio) = v.first() {
                                nfo.set_studio(studio.to_string());
                            }
                        }
                        "出演者" => {
                            for actor in v.iter() {
                                nfo.actors_mut().insert(actor.to_string());
                            }
                        }
                        "ジャンル" => {
                            for genre in v.iter() {
                                nfo.genres_mut().insert(genre.to_string());
                            }
                        }
                        "配信開始日" => {
                            if let Some(date) = v.first() {
                                nfo.set_premiered(date.to_string());
                            }
                        }
                        "収録時間" => {
                            if let Some(runtime) = v.first() {
                                let runtime: String =
                                    runtime.chars().filter(|c| c.is_ascii_digit()).collect();
                                let runtime: u32 = runtime.parse().unwrap_or_default();
                                nfo.set_runtime(runtime);
                            }
                        }
                        "平均評価" => {
                            if let Some(rating) = v.first() {
                                let rating: f64 = rating.parse().unwrap_or_default();
                                nfo.set_rating(rating);
                            }
                        }
                        _ => {}
                    }
                }

                v.clear();
            }
        }

        Ok((poster, fanart))
    }
}
