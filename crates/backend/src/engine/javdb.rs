use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use scraper::Html;
use tracing::{info, warn};

use crate::{image_loader, select, Engine, Info, Video};

pub struct Javdb {
    client: Arc<Client>,
}
image_loader!(Javdb);

impl Javdb {
    const HOST: &'static str = "https://javdb.com";

    pub fn new(client: Arc<Client>) -> Javdb {
        Javdb { client }
    }

    async fn find_item(&self, video: &Video) -> anyhow::Result<Option<String>> {
        select!(
            items: "body > section > div > div.movie-list.h.cols-4.vcols-8 > div.item > a",
            id: "div.video-title > strong"
        );
        let url = format!("https://javdb.com/search?q={}&f=all", video.id());
        let res = self.client.get(url).send().await?.text().await?;
        let doc = Html::parse_document(&res);
        let Some(item) = doc.select(&selectors().items).find(|item| {
            item.select(&selectors().id)
                .next()
                .map(|item| video.matches(&item.inner_html()))
                .unwrap_or(false)
        }) else {
            return Ok(None);
        };
        let Some(href) = item
            .attr("href")
            .map(|href| format!("{}{}", Javdb::HOST, href))
        else {
            return Ok(None);
        };

        Ok(Some(href))
    }

    async fn load_info(&self, href: &str, info: &mut Info) -> anyhow::Result<Option<String>> {
        select!(
            title: "body > section > div > div.video-detail > h2",
            fanart: "body > section > div > div.video-detail > div.video-meta-panel > div > div.column.column-video-cover > a > img",
            tag: "body > section > div > div.video-detail > div.video-meta-panel > div > div:nth-child(2) > nav > div.panel-block"
        );
        let res = self.client.get(href).send().await?.text().await?;
        let doc = Html::parse_document(&res);
        let fanart = doc
            .select(&selectors().fanart)
            .next()
            .and_then(|img| img.attr("src").map(|src| src.to_string()));
        if let Some(title) = doc.select(&selectors().title).next().map(|title| {
            title
                .text()
                .flat_map(|text| text.trim().chars())
                .collect::<String>()
        }) {
            info.title(title);
        }
        let tags = doc
            .select(&selectors().tag)
            .map(|tag| tag.text().flat_map(|tag| tag.chars()).collect::<String>())
            .collect::<Vec<String>>();
        let tags = tags
            .iter()
            .flat_map(|tag| tag.split_once(':').map(|(k, v)| (k.trim(), v.trim())))
            .collect::<Vec<(&str, &str)>>();
        for (k, v) in tags {
            match k {
                "日期" => info.premiered(v.to_string()),
                "時長" => {
                    let runtime = v
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                        .unwrap_or(0);
                    info.runtime(runtime)
                }
                "導演" => info.director(v.to_string()),
                "片商" => info.studio(v.to_string()),
                "評分" => {
                    let rating = v
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect::<String>()
                        .parse::<f64>()
                        .unwrap_or(0.0);
                    info.rating(rating * 2.0);
                }
                "類別" => {
                    let genres = v
                        .split(',')
                        .map(|genre| genre.trim().to_string())
                        .collect::<Vec<String>>();
                    info.genres(genres);
                }
                "演員" => {
                    let actors = v
                        .lines()
                        .map(|line| {
                            line.trim()
                                .trim_end_matches('♂')
                                .trim_end_matches('♀')
                                .to_string()
                        })
                        .collect::<Vec<String>>();
                    info.actors(actors);
                }
                _ => {}
            }
        }

        Ok(fanart)
    }
}

#[async_trait]
impl Engine for Javdb {
    async fn search(&self, video: &Video) -> anyhow::Result<Info> {
        info!("search {} in Javdb", video.id());
        let mut info = Info::default();
        let Some(href) = self.find_item(video).await? else {
            warn!("{} not found in Javdb", video.id());
            return Ok(info);
        };
        if let Some(fanart) = self.load_info(&href, &mut info).await? {
            let fanart = self.load_img(&fanart).await?;
            info.fanart(fanart);
        }

        info!("{} found in Javdb", video.id());
        Ok(info)
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _, _) => false,
            Video::Normal(_, _, _) => true,
        }
    }
}
