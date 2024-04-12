use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::Client;
use scraper::Html;
use tracing::info;
use video::Video;

use crate::{Engine, Info};

pub struct Javdb {
    client: Arc<Client>,
}

impl Javdb {
    const HOST: &str = "https://javdb.com";

    pub fn new(client: Arc<Client>) -> Javdb {
        Javdb { client }
    }

    async fn find_item(&self, key: &str) -> Result<()> {
        let url = format!("https://javdb.com/search?q={}&f=all", key);
        let res = self.client.get(url).send().await?.text().await?;
        let doc = Html::parse_document(&res);

        Ok(())
    }
}

#[async_trait]
impl Engine for Javdb {
    async fn search(&self, key: &str) -> Result<Info> {
        info!("search {key} in Javdb");

        Ok(Info::default()
            .id(key.to_string())
            // .actors(vec!["he".to_string(), "she".to_string()])
            .director("dir".to_string())
            .genres(vec!["gen".to_string(), "res".to_string()])
            .plot("plot".to_string())
            .premiered("date".to_string())
            .rating(8.8)
            .runtime(16)
            .studio("studio".to_string())
            .title("title".to_string()))
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _) => true,
            Video::Normal(_, _) => true,
        }
    }
}
