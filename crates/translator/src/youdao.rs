use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bon::bon;
use log::info;
use reqwest::{Client, Proxy};
use serde::Deserialize;
use sha256::digest;
use uuid::Uuid;

use super::Handler;

pub struct Youdao {
    client: Client,
    key: String,
    secret: String,
}

#[bon]
impl Youdao {
    #[builder]
    pub fn new(
        key: impl Into<String>,
        secret: impl Into<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Youdao> {
        let mut client_builder = Client::builder().timeout(timeout);
        if let Some(url) = proxy {
            let proxy = Proxy::all(&url).with_context(|| format!("set proxy to {url}"))?;
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder
            .build()
            .with_context(|| "build reqwest client")?;
        let youdao = Youdao {
            client,
            key: key.into(),
            secret: secret.into(),
        };

        Ok(youdao)
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
}

#[async_trait]
impl Handler for Youdao {
    fn name(&self) -> &'static str {
        "youdao"
    }

    async fn translate(&self, content: &str) -> Result<String> {
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

        let url = "https://openapi.youdao.com/api";
        let from = "auto";
        let to = "zh-CHS";
        let salt = Uuid::new_v4().to_string();
        let cur_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .with_context(|| "get current time stamp")?
            .as_secs()
            .to_string();
        let sign = self.concat_sign(content, &salt, &cur_time);

        let res = self
            .client
            .get(url)
            .query(&[
                ("q", content),
                ("from", from),
                ("to", to),
                ("appKey", &self.key),
                ("salt", &salt),
                ("sign", &sign),
                ("signType", "v3"),
                ("curtime", &cur_time),
                ("strict", "true"),
            ])
            .send()
            .await
            .with_context(|| format!("send to {url}"))?
            .json::<Response>()
            .await
            .with_context(|| format!("decode to json from {url}"))?;

        if res.code != "0" {
            info!("translate failed, code: {}", res.code);
            bail!("translate failed, code: {}", res.code);
        }

        let Some(translated) = res.translation.map(|trans| trans.join("\n")) else {
            info!("translate failed, no response");
            bail!("translate failed, no response");
        };

        Ok(translated)
    }
}
