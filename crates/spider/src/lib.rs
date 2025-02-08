mod avsox;
mod jav321;
mod javdb;
mod missav;
mod subtitle_cat;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use avsox::Avsox;
use config::Config;
use jav321::Jav321;
use javdb::Javdb;
use log::error;
use missav::Missav;
use nfo::Nfo;
use subtitle_cat::SubtitleCat;
use video::VideoType;

#[async_trait]
trait Finder: Send + Sync {
    async fn find(&self, key: VideoType) -> Result<Nfo>;
}

pub struct Spider {
    finders: Vec<Arc<dyn Finder>>,
}

impl Spider {
    pub fn new(config: &Config) -> Result<Spider> {
        let timeout = Duration::from_secs(config.network.timeout);
        let proxy = &config.network.proxy;
        let url = &config.url;
        let finders: Vec<Arc<dyn Finder>> = vec![
            Arc::new(Missav::new(timeout, proxy.clone())?),
            Arc::new(
                Avsox::builder()
                    .maybe_base_url(url.avsox.clone())
                    .timeout(timeout)
                    .maybe_proxy(proxy.clone())
                    .build()?,
            ),
            Arc::new(SubtitleCat::new(timeout, proxy.clone())?),
            Arc::new(Jav321::new(timeout, proxy.clone())?),
            Arc::new(
                Javdb::builder()
                    .maybe_base_url(url.javdb.clone())
                    .timeout(timeout)
                    .maybe_proxy(proxy.clone())
                    .build()?,
            ),
        ];

        let spider = Spider { finders };
        Ok(spider)
    }

    pub async fn find(&self, key: VideoType) -> Result<Nfo> {
        let mut tasks = Vec::new();
        for finder in self.finders.iter() {
            let finder = finder.clone();
            let key = key.clone();
            let task = tokio::spawn(async move { finder.find(key).await });
            tasks.push(task);
        }

        let mut nfo = Nfo::new(key.name());
        nfo.set_mpaa("NC-17".to_string());
        for task in tasks {
            match task.await? {
                Ok(found_nfo) => nfo.merge(found_nfo),
                Err(err) => error!("{err}"),
            }
        }

        Ok(nfo)
    }
}
