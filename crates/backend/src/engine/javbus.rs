use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Client;
use scraper::selectable::Selectable;
use scraper::Html;
use tracing::{info, warn};

use crate::video::Video;
use crate::{select, Engine, Info};

pub struct Javbus {
    client: Arc<Client>,
    headers: HeaderMap,
}

impl Javbus {
    const HOST: &'static str = "https://www.javbus.com";

    pub fn new(client: Arc<Client>) -> Javbus {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("www.javbus.com"));

        Javbus { client, headers }
    }

    async fn find_item(&self, video: &Video) -> Result<Option<(String, String)>> {
        select!(
            (items: "#waterfall > div.item > a.movie-box"),
            (poster: "div.photo-frame > img"),
            (id: "div.photo-info > span > date:nth-child(3)")
        );
        let url = format!("{}/search/{}&type=&parent=ce", Javbus::HOST, video.id());
        let res = self
            .client
            .get(url)
            .headers(self.headers.clone())
            .send()
            .await?
            .text()
            .await?;
        let doc = Html::parse_document(&res);
        let Some(item) = doc.select(&selectors().items).find(|item| {
            item.select(&selectors().id)
                .next()
                .map(|item| video.matches(&item.inner_html()))
                .unwrap_or(false)
        }) else {
            return Ok(None);
        };
        let Some(href) = item.attr("href") else {
            return Ok(None);
        };
        let Some(poster) = item.select(&selectors().poster).next().and_then(|img| {
            img.attr("src")
                .map(|src| format!("{}{}", Javbus::HOST, src))
        }) else {
            return Ok(None);
        };

        Ok(Some((href.to_string(), poster)))
    }

    async fn load_info(&self, href: &str, mut info: Info) -> Result<(Option<String>, Info)> {
        select!(
            (title: "body > div.container > h3"),
            (fanart: "body > div.container > div.row.movie > div.col-md-9.screencap > a > img"),
            (tag: "body > div.container > div.row.movie > div.col-md-3.info > p")
        );
        let res = self
            .client
            .get(href)
            .headers(self.headers.clone())
            .send()
            .await?
            .text()
            .await?;
        let doc = Html::parse_document(&res);
        let fanart = doc.select(&selectors().fanart).next().and_then(|fanart| {
            fanart
                .attr("src")
                .map(|src| format!("{}{}", Javbus::HOST, src))
        });
        if let Some(title) = doc
            .select(&selectors().title)
            .next()
            .map(|title| title.inner_html())
        {
            info = info.title(title);
        }
        let tags = doc
            .select(&selectors().tag)
            .map(|tag| tag.text().flat_map(|tag| tag.chars()).collect::<String>())
            .collect::<Vec<String>>();
        let pairs = Javbus::parse_tags(&tags);
        for (k, v) in pairs {
            match k {
                "發行日期" => info = info.premiered(v.to_string()),
                "長度" => {
                    info = info.runtime(
                        v.chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse::<u32>()
                            .unwrap_or(0),
                    )
                }
                "導演" => info = info.director(v.to_string()),
                "製作商" => info = info.studio(v.to_string()),
                "類別" => {
                    info = info.genres(
                        v.lines()
                            .map(|line| line.trim())
                            .filter(|line| !line.contains("多選提交"))
                            .map(|line| line.to_string())
                            .collect(),
                    )
                }
                "演員" => {
                    info = info.actors(v.lines().map(|line| line.trim().to_string()).collect())
                }
                _ => {}
            }
        }

        Ok((fanart, info))
    }

    fn parse_tags(tags: &[String]) -> Vec<(&str, &str)> {
        let len = tags.len();
        let mut i = 0;
        let mut pairs = Vec::new();
        let mut key = "";
        while i < len {
            let tag = tags[i].trim();
            if tag.ends_with(':') {
                key = tag;
            } else {
                match tag.split_once(':') {
                    Some((k, v)) => pairs.push((k.trim(), v.trim())),
                    None => {
                        pairs.push((key.trim_end_matches(':'), tag));
                    }
                }
            }

            i += 1;
        }

        pairs
    }

    async fn load_img(&self, url: &str) -> Result<Vec<u8>> {
        Ok(self.client.get(url).send().await?.bytes().await?.to_vec())
    }
}

#[async_trait]
impl Engine for Javbus {
    async fn search(&self, video: &Video) -> Result<Info> {
        info!("search {} in Javbus", video.id());
        let info = Info::default();
        let Some((href, poster)) = self.find_item(video).await? else {
            warn!("{} not found in Javbus", video.id());
            return Ok(info);
        };
        let (fanart, mut info) = self.load_info(&href, info).await?;
        let poster = self.load_img(&poster).await?;
        info = info.poster(poster);
        if let Some(fanart) = fanart {
            let fanart = self.load_img(&fanart).await?;
            info = info.fanart(fanart);
        }

        info!("{} found in Javbus", video.id());
        Ok(info)
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _) => false,
            Video::Normal(_, _) => true,
        }
    }
}
