use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::Client;
use video::Video;

mod engine;
mod info;

use engine::{Javbus, Javdb};
use info::Info;

pub struct Backend {
    engines: Vec<Box<dyn Engine>>,
}

impl Backend {
    pub fn new() -> Result<Backend> {
        let client = Client::builder().build()?;
        let client = Arc::new(client);
        let engines: Vec<Box<dyn Engine>> = vec![
            Box::new(Javbus::new(client.clone())),
            Box::new(Javdb::new(client)),
        ];

        Ok(Backend { engines })
    }

    pub async fn search(&self, video: &Video) -> Option<Info> {
        let mut info = Info::new();
        for engine in self.engines.iter() {
            if engine.could_solve(video) {
                if let Ok(new_info) = engine.search(video.id()).await {
                    info.merge(new_info);
                }
            }
        }

        info.check()
    }
}

#[async_trait]
pub trait Engine {
    async fn search(&self, key: &str) -> Result<Info>;
    fn could_solve(&self, video: &Video) -> bool;
}
