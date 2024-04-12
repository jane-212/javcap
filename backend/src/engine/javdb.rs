use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::Client;
use video::Video;

use crate::{Engine, Info};

pub struct Javdb {
    client: Arc<Client>,
}

impl Javdb {
    pub fn new(client: Arc<Client>) -> Javdb {
        Javdb { client }
    }
}

#[async_trait]
impl Engine for Javdb {
    async fn search(&self, key: &str) -> Result<Info> {
        todo!()
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _) => true,
            Video::Normal(_, _) => true,
        }
    }
}
