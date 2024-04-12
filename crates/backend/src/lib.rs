use std::sync::Arc;

use async_trait::async_trait;
use error::Result;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Proxy,
};
use video::Video;

mod engine;
mod info;

use engine::{Javbus, Javdb};
use info::Info;

pub struct Backend {
    engines: Vec<Box<dyn Engine>>,
}

impl Backend {
    pub fn new(proxy: &str) -> Result<Backend> {
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
            .proxy(Proxy::https(proxy)?)
            .build()?;
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

#[macro_export]
macro_rules! select {
    ($(($k:ident: $v: expr)),*) => {
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
