use std::sync::Arc;

use async_trait::async_trait;
use macros::Engine;
use reqwest::Client;
use scraper::Html;

use crate::select;
use crate::task::video::{Engine, Info, VideoParser};

#[derive(Engine)]
#[engine(image_loader)]
pub struct Javlib {
    client: Arc<Client>,
}

impl Javlib {
    pub fn new(client: Arc<Client>) -> Javlib {
        Javlib { client }
    }

    async fn find_item(&self, video: &VideoParser) -> anyhow::Result<Option<String>> {
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

    fn load_info(res: String, info: &mut Info) -> anyhow::Result<Option<String>> {
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
            .map(|title| title.inner_html().trim().to_string())
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
}

#[async_trait]
impl Engine for Javlib {
    async fn search(&self, video: &VideoParser) -> anyhow::Result<Info> {
        let mut info = Info::default();
        let Some(res) = self.find_item(video).await? else {
            return Ok(info);
        };
        if let Some(fanart) = Javlib::load_info(res, &mut info)? {
            let fanart = self.load_img(&fanart).await?;
            info.fanart(fanart);
        }

        Ok(info)
    }

    fn could_solve(&self, video: &VideoParser) -> bool {
        match video {
            VideoParser::FC2(_, _, _) => false,
            VideoParser::Normal(_, _, _) => true,
        }
    }

    fn id(&self) -> &'static str {
        self.key()
    }
}
