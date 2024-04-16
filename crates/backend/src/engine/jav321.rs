use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::Client;
use scraper::Html;
use tracing::{info, warn};

use crate::{select, Engine, Info, Video};

pub struct Jav321 {
    client: Arc<Client>,
}

impl Jav321 {
    pub fn new(client: Arc<Client>) -> Jav321 {
        Jav321 { client }
    }

    async fn find_item(&self, video: &Video) -> Result<Option<String>> {
        select!(
            info: "body > div.row > div.col-md-10.col-md-offset-1.col-xs-10 > div.alert.alert-danger"
        );
        let url = "https://tw.jav321.com/search";
        let mut form = HashMap::new();
        form.insert("sn", video.id());
        let res = self
            .client
            .post(url)
            .form(&form)
            .send()
            .await?
            .text()
            .await?;
        let doc = Html::parse_document(&res);
        if doc.select(&selectors().info).next().is_some() {
            return Ok(None);
        }

        Ok(Some(res))
    }

    fn load_info(res: String, info: &mut Info) -> Result<Option<String>> {
        select!(
            title: "body > div:nth-child(3) > div.col-md-7.col-md-offset-1.col-xs-12 > div:nth-child(1) > div.panel-heading > h3",
            poster: "body > div:nth-child(3) > div.col-md-7.col-md-offset-1.col-xs-12 > div:nth-child(1) > div.panel-body > div:nth-child(1) > div.col-md-3 > img",
            plot: "body > div:nth-child(3) > div.col-md-7.col-md-offset-1.col-xs-12 > div:nth-child(1) > div.panel-body > div:nth-child(3) > div",
            tag: "body > div:nth-child(3) > div.col-md-7.col-md-offset-1.col-xs-12 > div:nth-child(1) > div.panel-body > div:nth-child(1) > div.col-md-9"
        );
        let doc = Html::parse_document(&res);
        if let Some(title) = doc
            .select(&selectors().title)
            .next()
            .and_then(|title| title.text().next().map(|title| title.to_string()))
        {
            info.title(title);
        }
        let poster = doc
            .select(&selectors().poster)
            .next()
            .and_then(|img| img.attr("src").map(|src| src.to_string()));
        if let Some(plot) = doc
            .select(&selectors().plot)
            .next()
            .and_then(|plot| plot.text().next().map(|plot| plot.to_string()))
        {
            info.plot(plot);
        }
        if let Some(tags) = doc
            .select(&selectors().tag)
            .next()
            .map(|tag| tag.text().map(|tag| tag.trim()))
        {
            let lines = tags.collect::<Vec<&str>>();
            let tags = Jav321::parse_tags(&lines);
            for (k, v) in tags {
                match k {
                    "女優" => {
                        let mut actors = Vec::new();
                        for actor in v.split(' ') {
                            if actor.is_empty() {
                                continue;
                            }

                            actors.push(actor.to_string());
                        }

                        info.actors(actors);
                    }
                    "發行商" => info.studio(v.to_string()),
                    "發行日期" => info.premiered(v.to_string()),
                    "播放時長" => info.runtime(
                        v.chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse::<u32>()
                            .unwrap_or(0),
                    ),
                    _ => {}
                }
            }
        }

        Ok(poster)
    }

    fn parse_tags<'a>(lines: &'a [&'a str]) -> Vec<(&str, &str)> {
        let mut tags = Vec::new();
        let len = lines.len();
        let mut i = 0;
        let mut key = "";
        let mut is_value = false;
        while i < len {
            let line = lines[i];
            match line {
                ":" => is_value = true,
                "評分" => i += 1,
                "贊" => i += 1,
                "女優" => key = "女優",
                "發行商" => key = "發行商",
                "番號" => key = "番號",
                "發行日期" => key = "發行日期",
                "播放時長" => key = "播放時長",
                _ => {
                    if is_value {
                        tags.push((key, line));
                        is_value = false;
                    } else if line.starts_with(':') {
                        let line = line.trim_start_matches(':').trim();
                        tags.push((key, line));
                    }
                }
            }
            i += 1;
        }

        tags
    }

    async fn load_img(&self, url: &str) -> Result<Vec<u8>> {
        Ok(self.client.get(url).send().await?.bytes().await?.to_vec())
    }
}

#[async_trait]
impl Engine for Jav321 {
    async fn search(&self, video: &Video) -> Result<Info> {
        info!("search {} in Jav321", video.id());
        let mut info = Info::default();
        let Some(res) = self.find_item(video).await? else {
            warn!("{} not found in Jav321", video.id());
            return Ok(info);
        };
        if let Some(poster) = Jav321::load_info(res, &mut info)? {
            let poster = self.load_img(&poster).await?;
            info.poster(poster);
        }

        info!("{} found in Jav321", video.id());
        Ok(info)
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _) => false,
            Video::Normal(_, _) => true,
        }
    }
}
