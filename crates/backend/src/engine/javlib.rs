use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::Client;
use scraper::Html;
use tracing::{info, warn};

use crate::{select, Engine, Info, Video};

pub struct Javlib {
    client: Arc<Client>,
}

impl Javlib {
    pub fn new(client: Arc<Client>) -> Javlib {
        Javlib { client }
    }

    async fn find_item(&self, video: &Video) -> Result<Option<String>> {
        select!(
            info: "#rightcolumn > p > em"
        );
        let url = format!(
            "https://www.javlibrary.com/cn/vl_searchbyid.php?keyword={}",
            video.id()
        );
        let res = self.client.get(url).send().await?.text().await?;
        let doc = Html::parse_document(&res);
        if doc.select(&selectors().info).next().is_some() {
            return Ok(None);
        }

        Ok(Some(res))
    }

    fn load_info(res: String, info: &mut Info) -> Result<Option<String>> {
        select!(
            title: "#video_title > h3 > a",
            fanart: "#video_jacket_img",
            tag: "#video_info > div.item",
            genre: "#video_genres > table > tbody > tr > td.text > span.genre > a"
        );
        let doc = Html::parse_document(&res);
        if let Some(title) = doc
            .select(&selectors().title)
            .next()
            .map(|title| title.inner_html())
        {
            info.title(title);
        }
        let fanart = doc
            .select(&selectors().fanart)
            .next()
            .and_then(|fanart| fanart.attr("src").map(|src| src.to_string()));
        let tags = doc
            .select(&selectors().tag)
            .map(|tag| tag.text().flat_map(|tag| tag.chars()).collect::<String>())
            .collect::<Vec<String>>();
        let tags = tags
            .iter()
            .flat_map(|tag| {
                tag.trim()
                    .split_once(':')
                    .map(|(k, v)| (k.trim(), v.trim()))
            })
            .collect::<Vec<(&str, &str)>>();
        for (k, v) in tags {
            match k {
                "发行日期" => info.premiered(v.to_string()),
                "长度" => info.runtime(
                    v.chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                        .unwrap_or(0),
                ),
                "导演" => info.director(v.to_string()),
                "制作商" => info.studio(v.to_string()),
                "使用者评价" => info.rating(
                    v.chars()
                        .filter(|c| c.is_ascii_digit() || *c == '.')
                        .collect::<String>()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ),
                "演员" => info.actors(vec![v.to_string()]),
                _ => {}
            }
        }
        let genres = doc
            .select(&selectors().genre)
            .map(|genre| genre.inner_html())
            .collect::<Vec<String>>();
        info.genres(genres);

        Ok(fanart)
    }

    async fn load_img(&self, url: &str) -> Result<Vec<u8>> {
        Ok(self.client.get(url).send().await?.bytes().await?.to_vec())
    }
}

#[async_trait]
impl Engine for Javlib {
    async fn search(&self, video: &Video) -> Result<Info> {
        info!("search {} in Javlib", video.id());
        let mut info = Info::default();
        let Some(res) = self.find_item(video).await? else {
            warn!("{} not found in Javlib", video.id());
            return Ok(info);
        };
        if let Some(fanart) = Javlib::load_info(res, &mut info)? {
            let fanart = self.load_img(&fanart).await?;
            info.fanart(fanart);
        }

        info!("{} found in Javlib", video.id());
        Ok(info)
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _) => false,
            Video::Normal(_, _) => true,
        }
    }
}
