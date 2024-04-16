use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use engine::{Avsox, Jav321, Javbus, Javdb, Javlib, Mgstage};
use error::{Error, Result};
use info::Info;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Proxy,
};
use tracing::warn;
use translate::Translate;
use video::Video;

mod engine;
mod info;
mod translate;
pub mod video;

pub struct Backend {
    engines: Vec<Arc<Box<dyn Engine>>>,
    translate: Translate,
    client: Arc<Client>,
}

impl Backend {
    pub fn new(proxy: &str, timeout: u64) -> Result<Backend> {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4.1 Safari/605.1.15"));
        headers.insert(
            header::ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br"),
        );
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static("zh-CN,zh-Hans;q=0.9"),
        );
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(timeout))
            .proxy(Proxy::https(proxy)?)
            .build()?;
        let client = Arc::new(client);
        let engines: Vec<Arc<Box<dyn Engine>>> = vec![
            Arc::new(Box::new(Javbus::new(client.clone()))),
            Arc::new(Box::new(Javdb::new(client.clone()))),
            Arc::new(Box::new(Javlib::new(client.clone()))),
            Arc::new(Box::new(Jav321::new(client.clone()))),
            Arc::new(Box::new(Avsox::new(client.clone()))),
            Arc::new(Box::new(Mgstage::new(client.clone()))),
        ];
        let translate = Translate::new(client.clone());

        Ok(Backend {
            engines,
            translate,
            client,
        })
    }

    pub async fn ping(&self, url: &str) -> Result<()> {
        let status = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|_| Error::Proxy)?
            .status();
        if !status.is_success() {
            return Err(Error::Proxy);
        }

        Ok(())
    }

    pub async fn search(&mut self, video: &Video) -> Option<Info> {
        let mut info = Info::new(video.id().to_string());
        let mut handles = Vec::with_capacity(self.engines.len());
        for engine in self.engines.clone() {
            if engine.could_solve(video) {
                let video = video.clone();
                let handle = tokio::spawn(async move { engine.search(&video).await });
                handles.push(handle);
            }
        }
        for handle in handles {
            if let Ok(new_info) = handle.await {
                match new_info {
                    Ok(new_info) => {
                        info.merge(new_info);
                    }
                    Err(err) => warn!("{err}"),
                }
            }
        }
        self.translate.translate(&mut info).await.ok();

        #[cfg(debug_assertions)]
        info.show_info("SUMARY");
        info.check(video)
    }
}

#[async_trait]
pub trait Engine: Send + Sync {
    async fn search(&self, video: &Video) -> Result<Info>;
    fn could_solve(&self, video: &Video) -> bool;
}

#[macro_export]
macro_rules! select {
    ($($k:ident: $v: expr),*) => {
        struct Selectors {
            $(pub $k: scraper::Selector),*
        }

        impl Selectors {
            fn new() -> Self {
                Self {
                    $($k: scraper::Selector::parse($v).expect(&format!("parse {} failed",stringify!($k)))),*
                }
            }
        }

        fn selectors() -> &'static Selectors {
            static SELECTORS: std::sync::OnceLock<Selectors> = std::sync::OnceLock::new();
            SELECTORS.get_or_init(Selectors::new)
        }
    };
}
