use super::Translator;
use crate::task::video::info::Info;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};

pub struct AppWorld {
    client: Arc<Client>,
    last_tick: Instant,
}

impl AppWorld {
    const INTERVAL: Duration = Duration::from_secs(2);

    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            last_tick: Instant::now(),
        }
    }

    async fn translate_single(&mut self, text: impl AsRef<str>) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Response {
            code: i32,
            data: Option<String>,
            msg: String,
        }

        let text = text.as_ref();

        if self.last_tick.elapsed() < Self::INTERVAL {
            sleep(Self::INTERVAL).await;
        }
        self.last_tick = Instant::now();

        let url = "https://translate.appworlds.cn";
        let from = "auto";
        let to = "zh-CN";

        let res = self
            .client
            .get(url)
            .query(&[("text", text), ("from", from), ("to", to)])
            .send()
            .await?
            .json::<Response>()
            .await?;

        if res.code != 200 || res.data.is_none() {
            anyhow::bail!(res.msg);
        }

        Ok(res.data.unwrap_or("".to_string()))
    }
}

#[async_trait]
impl Translator for AppWorld {
    async fn translate<'a>(&mut self, info: &mut Info) -> anyhow::Result<()> {
        let title = info.get_title();
        if !title.is_empty() {
            let translated_title = self.translate_single(title).await?;
            info.title(translated_title);
        }

        let plot = info.get_plot();
        if !plot.is_empty() {
            let translated_plot = self.translate_single(plot).await?;
            info.plot(translated_plot);
        }

        Ok(())
    }
}
