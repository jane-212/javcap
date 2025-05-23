mod airav;
mod avsox;
mod cable;
mod fc2ppv_db;
mod hbox;
mod jav321;
mod javdb;
mod missav;
mod porny;
mod subtitle_cat;
mod the_porn_db;

use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use airav::Airav;
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use avsox::Avsox;
use cable::Cable;
use config::Config;
use fc2ppv_db::Fc2ppvDB;
use hbox::Hbox;
use jav321::Jav321;
use javdb::Javdb;
use log::{error, warn};
use missav::Missav;
use nfo::{Country, Nfo};
use porny::Porny;
use subtitle_cat::SubtitleCat;
use the_porn_db::ThePornDB;
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

        macro_rules! spider {
            ($s:ty, $u:expr, $m:expr) => {
                Arc::new(
                    <$s>::builder()
                        .maybe_base_url($u)
                        .timeout(timeout)
                        .maybe_proxy(proxy.clone())
                        .build()
                        .with_context(|| concat!("build ", $m))?,
                )
            };
        }

        let mut finders: Vec<Arc<dyn Finder>> = vec![
            spider!(Airav, url.airav.clone(), "airav"),
            spider!(Avsox, url.avsox.clone(), "avsox"),
            spider!(Cable, url.cable.clone(), "cable"),
            spider!(Fc2ppvDB, url.fc2ppv_db.clone(), "fc2ppv db"),
            spider!(Hbox, url.hbox.clone(), "hbox"),
            spider!(Jav321, url.jav321.clone(), "jav321"),
            spider!(Javdb, url.javdb.clone(), "javdb"),
            spider!(Missav, url.missav.clone(), "missav"),
            spider!(Porny, url.porny.clone(), "91 porny"),
            spider!(SubtitleCat, url.subtitle_cat.clone(), "subtitle cat"),
        ];
        if let Some(ref key) = config.the_porn_db.key {
            finders.push(Arc::new(
                ThePornDB::builder()
                    .maybe_base_url(url.the_porn_db.clone())
                    .maybe_api_url(url.the_porn_db_api.clone())
                    .key(key)
                    .timeout(timeout)
                    .maybe_proxy(proxy.clone())
                    .build()
                    .with_context(|| "build the porn db")?,
            ));
        }

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

fn which_country(key: &VideoType) -> Country {
    match key {
        VideoType::Jav(id, _) => match id.as_str() {
            "MD" | "LY" | "MDHG" | "MSD" | "SZL" | "MDSR" | "MDCM" | "PCM" | "YCM" | "KCM"
            | "PMX" | "PM" | "PMS" | "EMX" | "GDCM" | "XKTV" | "XKKY" | "XKG" | "XKVP" | "TM"
            | "TML" | "TMT" | "TMTC" | "TMW" | "JDYG" | "JD" | "JDKR" | "RAS" | "XSJKY"
            | "XSJYH" | "XSJ" | "IDG" | "FSOG" | "QDOG" | "TZ" | "DAD" => Country::China,
            _ => Country::Japan,
        },
        VideoType::Fc2(_) => Country::Japan,
        VideoType::Other(_) => Country::China,
    }
}

#[macro_export]
macro_rules! select {
    ($($k:ident: $v: expr)*) => {
        struct Selectors {
        $(
            $k: scraper::Selector,
        )*
        }

        impl Selectors {
            fn new() -> anyhow::Result<Selectors> {
                use anyhow::Context;

                let selectors = Selectors {
                $(
                    $k: scraper::Selector::parse($v)
                        .map_err(|e| anyhow::anyhow!("parse selector failed by {e}"))
                        .with_context(|| $v)
                        .with_context(|| stringify!($k))?,
                )*
                };

                Ok(selectors)
            }
        }
    };
}
