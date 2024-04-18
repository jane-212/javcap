use std::{collections::BTreeMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use avatar::Avatar;
use engine::{Avsox, Jav321, Javbus, Javdb, Javlib, Mgstage};
use info::Info;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Proxy,
};
use tracing::{info, warn};
use translate::Translator;
use video::Video;

mod avatar;
pub mod bar;
mod engine;
mod info;
mod translate;
pub mod video;

pub struct Backend {
    engines: Vec<Arc<Box<dyn Engine>>>,
    translate: Option<Box<dyn Translator>>,
    client: Arc<Client>,
    avatar: Avatar,
}

impl Backend {
    pub fn new(
        proxy: &str,
        timeout: u64,
        host: &str,
        api_key: &str,
        translate: &config::Translate,
    ) -> anyhow::Result<Backend> {
        let headers = {
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
            headers
        };
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(timeout))
            .proxy(
                Proxy::https(proxy)
                    .map_err(|_| anyhow::anyhow!("proxy {proxy} is not validate"))?,
            )
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
        let translate = match translate {
            config::Translate::Disable => None,
        };
        let avatar = Avatar::new(client.clone(), host.to_string(), api_key.to_string());

        Ok(Backend {
            engines,
            translate,
            client,
            avatar,
        })
    }

    pub async fn refresh_avatar(&self) -> anyhow::Result<()> {
        self.avatar.refresh().await
    }

    pub async fn ping(&self, url: &str) -> anyhow::Result<()> {
        let status = self.client.get(url).send().await?.status();
        if !status.is_success() {
            anyhow::bail!("ping url {url} failed");
        }

        Ok(())
    }

    pub async fn search(&mut self, video: &Video) -> Option<Info> {
        let mut info = Info::new(video.id().to_string());
        let mut handles = Vec::with_capacity(self.engines.len());
        for engine in self.engines.clone() {
            if engine.could_solve(video) {
                info!("search {} in {}", video.id(), engine.id());
                let id = engine.id().to_string();
                let video = video.clone();
                let handle = tokio::spawn(async move { engine.search(&video).await });
                handles.push((id, handle));
            }
        }
        for (id, handle) in handles {
            if let Ok(new_info) = handle.await {
                match new_info {
                    Ok(new_info) => {
                        info!("found {} in {}", video.id(), id);
                        info.merge(new_info);
                    }
                    Err(err) => warn!("{} not found in {id}, caused by {err}", video.id()),
                }
            }
        }

        #[cfg(debug_assertions)]
        info.show_info("SUMARY");
        info.check(video)
    }

    pub async fn translate(&mut self, info: &mut Info) -> anyhow::Result<()> {
        if let Some(ref translate) = self.translate {
            info!("translate");
            let mut text = BTreeMap::new();
            text.insert("title", info.get_title().to_string());
            text.insert("plot", info.get_plot().to_string());
            let res = translate.translate(text).await;
            if let Some(title) = res.get("title") {
                info.title(title.to_string());
            }
            if let Some(plot) = res.get("plot") {
                info.plot(plot.to_string());
            }
        }

        Ok(())
    }
}

#[async_trait]
pub trait Engine: Send + Sync {
    async fn search(&self, video: &Video) -> anyhow::Result<Info>;
    fn could_solve(&self, video: &Video) -> bool;
    fn id(&self) -> &'static str;
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
