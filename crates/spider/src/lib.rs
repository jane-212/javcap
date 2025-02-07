mod avsox;
mod missav;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use avsox::Avsox;
use config::Config;
use missav::Missav;
use nfo::Nfo;
use video::VideoType;

#[async_trait]
trait Finder: Send + Sync {
    async fn find(&self, key: VideoType) -> Result<Nfo>;
}

pub struct Spider {
    finders: Vec<Arc<dyn Finder>>,
}

impl Spider {
    // TODO:
    // avsox
    // jav321
    // javbus
    // javdb
    // javlib
    // mgstage
    pub fn new(config: &Config) -> Result<Spider> {
        let timeout = Duration::from_secs(config.network.timeout);
        let proxy = &config.network.proxy;
        let url = &config.url;
        let missav = Arc::new(Missav::new(timeout, proxy.clone())?);
        let avsox = Arc::new(Avsox::new(url.avsox.clone(), timeout, proxy.clone())?);
        let finders: Vec<Arc<dyn Finder>> = vec![missav, avsox];

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
            let Ok(found_nfo) = task.await? else {
                continue;
            };
            nfo.merge(found_nfo);
        }

        Ok(nfo)
    }
}
