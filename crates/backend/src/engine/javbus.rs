use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::Client;
use video::Video;

use crate::{Engine, Info};

pub struct Javbus {
    client: Arc<Client>,
}

impl Javbus {
    pub fn new(client: Arc<Client>) -> Javbus {
        Javbus { client }
    }
}

#[async_trait]
impl Engine for Javbus {
    async fn search(&self, key: &str) -> Result<Info> {
        
        
        Ok(Info::default()
            .id(key.to_string())
            .actors(vec!["he".to_string(), "she".to_string()])
            .director("dir".to_string())
            .genres(vec!["gen".to_string(), "res".to_string()])
            .plot("plot".to_string())
            .premiered("date".to_string())
            .rating(8.8)
            .runtime(160)
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
