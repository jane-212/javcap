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
        todo!()
    }

    fn could_solve(&self, video: &Video) -> bool {
        match video {
            Video::FC2(_, _) => true,
            Video::Normal(_, _) => true,
        }
    }
}
