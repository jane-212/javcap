use super::Translator;
use crate::task::video::info::Info;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use sha256::digest;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration, Instant};
use uuid::Uuid;

pub struct Youdao {
    client: Arc<Client>,
    last_tick: Instant,
    key: String,
    secret: String,
    salt: Uuid,
}

impl Youdao {
    const INTERVAL: Duration = Duration::from_secs(1);

    pub fn new(client: Arc<Client>, key: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            client,
            last_tick: Instant::now(),
            key: key.into(),
            secret: secret.into(),
            salt: Uuid::new_v4(),
        }
    }

    fn truncate(text: impl AsRef<str>) -> String {
        let text = text.as_ref();

        let len = text.chars().count();
        if len <= 20 {
            return text.to_owned();
        }

        format!(
            "{}{}{}",
            text.chars().take(10).collect::<String>(),
            len,
            text.chars().skip(len - 10).collect::<String>()
        )
    }

    fn concat_sign(
        &self,
        text: impl AsRef<str>,
        salt: impl AsRef<str>,
        cur_time: impl AsRef<str>,
    ) -> String {
        let text = text.as_ref();
        let salt = salt.as_ref();
        let cur_time = cur_time.as_ref();

        let not_signed = format!(
            "{}{}{}{}{}",
            self.key,
            Self::truncate(text),
            salt,
            cur_time,
            self.secret
        );

        digest(not_signed)
    }

    async fn translate_single(&mut self, text: impl AsRef<str>) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Response {
            #[serde(rename = "errorCode")]
            code: String,
            query: Option<String>,
            translation: Option<Vec<String>>,
            l: String,
            dict: Option<Dict>,
            webdict: Option<Dict>,
            #[serde(rename = "mTerminalDict")]
            m_terminal_dict: Option<Dict>,
            #[serde(rename = "tSpeakUrl")]
            t_speak_url: Option<String>,
            #[serde(rename = "speakUrl")]
            speak_url: Option<String>,
            #[serde(rename = "isDomainSupport")]
            is_domain_support: Option<String>,
            #[serde(rename = "requestId")]
            request_id: Option<String>,
            #[serde(rename = "isWord")]
            is_word: Option<bool>,
        }

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Dict {
            url: String,
        }

        let text = text.as_ref();

        if self.last_tick.elapsed() < Self::INTERVAL {
            sleep(Self::INTERVAL).await;
        }
        self.last_tick = Instant::now();

        let url = "https://openapi.youdao.com/api";
        let from = "auto";
        let to = "zh-CHS";
        let salt = self.salt.to_string();
        let cur_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .to_string();
        let sign = self.concat_sign(text, &salt, &cur_time);

        let res = self
            .client
            .get(url)
            .query(&[
                ("q", text),
                ("from", from),
                ("to", to),
                ("appKey", &self.key),
                ("salt", &salt),
                ("sign", &sign),
                ("signType", "v3"),
                ("curtime", &cur_time),
            ])
            .send()
            .await?
            .json::<Response>()
            .await?;

        if res.code != "0" {
            anyhow::bail!("translate failed, error code: {}", res.code);
        }

        Ok(res
            .translation
            .map(|trans| trans.join("\n"))
            .unwrap_or("".to_string()))
    }
}

#[async_trait]
impl Translator for Youdao {
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
