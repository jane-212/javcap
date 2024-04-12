use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Client;
use scraper::selectable::Selectable;
use scraper::Html;
use tracing::info;
use video::Video;

use crate::select;

use crate::{Engine, Info};

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

    async fn find_item(&self, key: &str) -> Result<Option<(String, String)>> {
        select!(
            (items: "#waterfall > div.item > a.movie-box"),
            (poster: "div.photo-frame > img"),
            (id: "div.photo-info > span > date:nth-child(3)")
        );
        let url = format!("{}/search/{}&type=&parent=ce", Javbus::HOST, key);
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
                .map(|item| item.inner_html() == key)
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

    async fn load_info(&self, href: &str, info: Info) -> Result<Info> {
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
        let Some(title) = doc
            .select(&selectors().title)
            .next()
            .map(|title| title.inner_html())
        else {
            return Ok(info);
        };
        let Some(fanart) = doc.select(&selectors().fanart).next().and_then(|fanart| {
            fanart
                .attr("src")
                .map(|src| format!("{}{}", Javbus::HOST, src))
        }) else {
            return Ok(info);
        };
        for tag in doc.select(&selectors().tag) {
            let tag = tag.text().flat_map(|tag| tag.chars()).collect::<String>();
            info!("tag: {}", tag); // TODO: remove it
        }

        Ok(info.title(title).fanart(fanart))
    }
}

// Ok(Info::default()
//     .id(key.to_string())
//     .actors(vec!["he".to_string(), "she".to_string()])
//     .director("dir".to_string())
//     .genres(vec!["gen".to_string(), "res".to_string()])
//     .plot("plot".to_string())
//     .premiered("date".to_string())
//     .rating(8.8)
//     .runtime(160)
//     .studio("studio".to_string())
//     .title("title".to_string()))

#[async_trait]
impl Engine for Javbus {
    async fn search(&self, key: &str) -> Result<Info> {
        info!("search {key} in Javbus");
        let info = Info::default();
        let Some((href, poster)) = self.find_item(key).await? else {
            info!("{key} not found in Javbus");
            return Ok(info);
        };
        let info = self.load_info(&href, info).await?;

        Ok(info.poster(poster))
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _) => true,
            Video::Normal(_, _) => true,
        }
    }
}
