use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::{info, warn};

use crate::{Engine, Info, Video};

pub struct Airav {
    client: Arc<Client>,
}

impl Airav {
    pub fn new(client: Arc<Client>) -> Airav {
        Airav { client }
    }

    async fn find_item(&self, video: &Video) -> Result<Option<(String, Option<String>)>> {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Response {
            offset: u32,
            count: u32,
            status: String,
            result: Vec<Result>,
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Result {
            vid: String,
            slug: Option<String>,
            name: String,
            url: String,
            view: u32,
            img_url: String,
            barcode: String,
        }
        let url = format!(
            "https://www.airav.wiki/api/video/list?lang=zh-TW&lng=zh-CN&search={}",
            video.id()
        );
        let res = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Response>()
            .await?;
        let Some(res) = res.result.into_iter().next() else {
            return Ok(None);
        };

        Ok(Some((
            format!(
                "https://www.airav.wiki/api/video/barcode/{}?lng=zh-CN",
                video.id()
            ),
            if res.img_url.is_empty() {
                None
            } else {
                Some(res.img_url)
            },
        )))
    }

    async fn load_info(&self, href: &str, mut info: Info) -> Result<Info> {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Response {
            count: u32,
            status: String,
            result: Result,
        }
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Result {
            id: u32,
            vid: String,
            slug: Option<String>,
            barcode: String,
            actors_name: String,
            name: String,
            img_url: String,
            other_images: Vec<String>,
            photo: Option<String>,
            publish_date: String,
            description: String,
            actors: Vec<Actor>,
            images: Vec<String>,
            tags: Vec<Tag>,
            factories: Vec<Tag>,
            maybe_like_videos: Vec<Maybe>,
            qc_url: String,
            view: u32,
            other_desc: Option<String>,
            video_url: VideoUrl,
        }
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Actor {
            name: String,
            name_cn: Option<String>,
            name_jp: String,
            name_en: String,
            id: String,
        }
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Tag {
            name: String,
        }
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Maybe {
            vid: String,
            slug: Option<String>,
            name: String,
            url: String,
            img_url: String,
            barcode: String,
        }
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct VideoUrl {
            url_cdn: String,
            url_hls_cdn: String,
        }
        let res = self
            .client
            .get(href)
            .send()
            .await?
            .json::<Response>()
            .await?;
        let res = res.result;
        info.title(res.name);
        info.premiered(res.publish_date);
        info.plot(res.description);
        info.actors(res.actors.into_iter().map(|actor| actor.name).collect());
        info.genres(res.tags.into_iter().map(|tag| tag.name).collect());
        if let Some(studio) = res.factories.into_iter().next().map(|fac| fac.name) {
            info.studio(studio);
        }

        Ok(info)
    }

    async fn load_img(&self, url: &str) -> Result<Vec<u8>> {
        Ok(self.client.get(url).send().await?.bytes().await?.to_vec())
    }
}

#[async_trait]
impl Engine for Airav {
    async fn search(&self, video: &Video) -> Result<Info> {
        info!("search {} in Avsox", video.id());
        let info = Info::default();
        let Some((href, fanart)) = self.find_item(video).await? else {
            warn!("{} not found in Airav", video.id());
            return Ok(info);
        };
        let mut info = self.load_info(&href, info).await?;
        if let Some(fanart) = fanart {
            let fanart = self.load_img(&fanart).await?;
            info.fanart(fanart);
        }

        info!("{} found in Airav", video.id());
        Ok(info)
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _) => false,
            Video::Normal(_, _) => true,
        }
    }
}
