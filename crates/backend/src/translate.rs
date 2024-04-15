use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use error::Result;
use reqwest::Client;
use serde::Deserialize;
use tokio::time::sleep;

use crate::info::Info;

pub struct Translate {
    client: Arc<Client>,
    timer: Instant,
}

impl Translate {
    const INTERVAL: Duration = Duration::from_secs(2);

    pub fn new(client: Arc<Client>) -> Translate {
        Translate {
            client,
            timer: Instant::now(),
        }
    }

    pub async fn translate(&mut self, info: &mut Info) -> Result<()> {
        if let Some(title) = self.get_trans(info.get_title()).await? {
            info.title(title);
        }
        if let Some(plot) = self.get_trans(info.get_plot()).await? {
            info.plot(plot);
        }

        Ok(())
    }

    async fn get_trans(&mut self, text: &str) -> Result<Option<String>> {
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct Response {
            code: u32,
            data: String,
            msg: String,
        }
        if self.timer.elapsed() < Translate::INTERVAL {
            sleep(Translate::INTERVAL).await;
        }
        self.timer = Instant::now();
        let url = Translate::get_url(text);
        let res = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Response>()
            .await?;
        if res.code == 200 {
            Ok(Some(res.data))
        } else {
            Ok(None)
        }
    }

    fn get_url(text: &str) -> String {
        format!(
            "https://translate.appworlds.cn?text={}&from=auto&to=zh-CN",
            text
        )
    }
}
