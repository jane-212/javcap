use crate::select;
use crate::task::video::{Engine, Info, VideoParser};
use async_trait::async_trait;
use macros::Engine;
use reqwest::Client;
use scraper::selectable::Selectable;
use scraper::Html;
use std::sync::Arc;

#[derive(Engine)]
#[engine(image_loader)]
pub struct Avsox {
    client: Arc<Client>,
}

impl Avsox {
    pub fn new(client: Arc<Client>) -> Avsox {
        Avsox { client }
    }

    async fn find_item(&self, video: &VideoParser) -> anyhow::Result<Option<String>> {
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

    async fn load_info(&self, href: &str, info: &mut Info) -> anyhow::Result<Option<String>> {
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
            .map(|title| title.inner_html().trim().to_string())
        {
            info.title(title);
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
                "发行时间" => info.premiered(v.to_string()),
                "长度" => info.runtime(
                    v.chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                        .unwrap_or(0),
                ),
                "制作商" => info.studio(v.to_string()),
                "系列" => info.director(v.to_string()),
                _ => {}
            }
        }

        Ok(fanart)
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
}

#[async_trait]
impl Engine for Avsox {
    async fn search(&self, video: &VideoParser) -> anyhow::Result<Info> {
        let mut info = Info::default();

        let Some(href) = self.find_item(video).await? else {
            return Ok(info);
        };

        if let Some(fanart) = self.load_info(&href, &mut info).await? {
            let fanart = self.load_img(&fanart).await?;
            info.fanart(fanart);
        }

        Ok(info)
    }

    fn support(&self, video: &VideoParser) -> bool {
        match video {
            VideoParser::FC2(_, _, _) => true,
            VideoParser::Normal(_, _, _) => false,
        }
    }

    fn id(&self) -> &'static str {
        self.key()
    }
}
