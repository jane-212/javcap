mod airav;
mod avsox;
mod fc2ppv_db;
mod hbox;
mod jav321;
mod javdb;
mod missav;
mod subtitle_cat;

use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use airav::Airav;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use avsox::Avsox;
use config::Config;
use fc2ppv_db::Fc2ppvDB;
use hbox::Hbox;
use jav321::Jav321;
use javdb::Javdb;
use log::{error, warn};
use missav::Missav;
use nfo::Nfo;
use subtitle_cat::SubtitleCat;
use video::VideoType;

#[async_trait]
trait Finder: Send + Sync + Display {
    fn support(&self, key: &VideoType) -> bool;
    async fn find(&self, key: &VideoType) -> Result<Nfo>;
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
            Arc::new(Missav::new(timeout, proxy.clone()).with_context(|| "build missav")?),
            Arc::new(
                Avsox::builder()
                    .maybe_base_url(url.avsox.clone())
                    .timeout(timeout)
                    .maybe_proxy(proxy.clone())
                    .build()
                    .with_context(|| "build avsox")?,
            ),
            Arc::new(
                SubtitleCat::new(timeout, proxy.clone()).with_context(|| "build subtitle cat")?,
            ),
            Arc::new(Jav321::new(timeout, proxy.clone()).with_context(|| "build jav321")?),
            Arc::new(
                Javdb::builder()
                    .maybe_base_url(url.javdb.clone())
                    .timeout(timeout)
                    .maybe_proxy(proxy.clone())
                    .build()
                    .with_context(|| "build javdb")?,
            ),
            Arc::new(Hbox::new(timeout, proxy.clone()).with_context(|| "build hbox")?),
            Arc::new(Fc2ppvDB::new(timeout, proxy.clone()).with_context(|| "build fc2ppv db")?),
            Arc::new(Airav::new(timeout, proxy.clone()).with_context(|| "build airav")?),
        ];

        let spider = Spider { finders };
        Ok(spider)
    }

    pub async fn find(&self, key: VideoType) -> Result<Nfo> {
        let key = Arc::new(key);
        let mut tasks = Vec::new();
        for finder in self.finders.iter() {
            if !finder.support(&key) {
                warn!("finder {finder} not support {key}");
                continue;
            }

            let finder = finder.clone();
            let key = key.clone();
            let task = tokio::spawn(async move {
                finder
                    .find(&key)
                    .await
                    .with_context(|| format!("in finder {finder}"))
            });
            tasks.push(task);
        }

        let mut nfo = None;
        for task in tasks {
            match task.await? {
                Ok(found_nfo) => match nfo {
                    None => nfo = Some(found_nfo),
                    Some(ref mut nfo) => nfo.merge(found_nfo),
                },
                Err(err) => error!("could not find {key}, caused by {err:?}"),
            }
        }

        nfo.ok_or_else(|| anyhow!("could not find anything about {key} in all finders"))
    }
}
