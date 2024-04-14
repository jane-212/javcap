use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::Client;
use scraper::selectable::Selectable;
use scraper::Html;
use tracing::{info, warn};

use crate::video::Video;
use crate::{select, Engine, Info};

pub struct Avsox {
    client: Arc<Client>,
}

impl Avsox {
    pub fn new(client: Arc<Client>) -> Avsox {
        Avsox { client }
    }

    async fn find_item(&self, video: &Video) -> Result<Option<String>> {
        select!(
            item: "#waterfall > div.item",
            id: "a > div.photo-info > span > date:nth-child(3)",
            href: "a"
        );
        let url = format!("https://avsox.click/cn/search/{}", video.id());
        let res = self.client.get(url).send().await?.text().await?;
        let doc = Html::parse_document(&res);
        let Some(item) = doc.select(&selectors().item).find(|item| {
            item.select(&selectors().id)
                .next()
                .map(|id| video.matches(&id.inner_html()))
                .unwrap_or(false)
        }) else {
            return Ok(None);
        };
        let Some(href) = item
            .select(&selectors().href)
            .next()
            .and_then(|href| href.attr("href").map(|href| format!("https:{}", href)))
        else {
            return Ok(None);
        };

        Ok(Some(href))
    }

    async fn load_info(&self, href: &str, mut info: Info) -> Result<(Option<String>, Info)> {
        select!(
            title: "body > div.container > h3",
            fanart: "body > div.container > div.row.movie > div.col-md-9.screencap > a > img",
            tag: "body > div.container > div.row.movie > div.col-md-3.info > p"
        );
        let res = self.client.get(href).send().await?.text().await?;
        let doc = Html::parse_document(&res);
        if let Some(title) = doc
            .select(&selectors().title)
            .next()
            .map(|title| title.inner_html())
        {
            info = info.title(title);
        }
        let fanart = doc
            .select(&selectors().fanart)
            .next()
            .and_then(|img| img.attr("src").map(|src| src.to_string()));
        let tags = doc
            .select(&selectors().tag)
            .map(|tag| tag.text().flat_map(|text| text.chars()).collect::<String>())
            .collect::<Vec<String>>();
        let tags = Avsox::parse_tags(&tags);
        for (k, v) in tags {
            match k {
                "发行时间" => info = info.premiered(v.to_string()),
                "长度" => {
                    info = info.runtime(
                        v.chars()
                            .take_while(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse::<u32>()
                            .unwrap_or(0),
                    )
                }
                "制作商" => info = info.studio(v.to_string()),
                "系列" => info = info.director(v.to_string()),
                _ => {}
            }
        }

        Ok((fanart, info))
    }

    fn parse_tags(tags: &[String]) -> Vec<(&str, &str)> {
        let mut ret = Vec::new();
        let len = tags.len();
        let mut i = 0;
        let mut is_value = false;
        let mut key = "";
        while i < len {
            let tag = tags[i].as_str().trim();
            if tag.ends_with(':') {
                is_value = true;
                key = tag.trim_end_matches(':');
            } else if is_value {
                ret.push((key, tag));
                is_value = false;
            } else if let Some((k, v)) = tag.split_once(':') {
                ret.push((k.trim(), v.trim()))
            }

            i += 1;
        }

        ret
    }

    async fn load_img(&self, url: &str) -> Result<Vec<u8>> {
        Ok(self.client.get(url).send().await?.bytes().await?.to_vec())
    }
}

#[async_trait]
impl Engine for Avsox {
    async fn search(&self, video: &Video) -> Result<Info> {
        info!("search {} in Avsox", video.id());
        let info = Info::default();
        let Some(href) = self.find_item(video).await? else {
            warn!("{} not found in Avsox", video.id());
            return Ok(info);
        };
        let (fanart, mut info) = self.load_info(&href, info).await?;
        if let Some(fanart) = fanart {
            let fanart = self.load_img(&fanart).await?;
            info = info.fanart(fanart);
        }

        info!("{} found in Avsox", video.id());
        Ok(info)
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _) => true,
            Video::Normal(_, _) => false,
        }
    }
}
