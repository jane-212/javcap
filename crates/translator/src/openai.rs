use std::time::Duration;

use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
};
use async_openai::Client;
use async_trait::async_trait;
use bon::bon;
use indoc::formatdoc;
use reqwest::Proxy;

use super::Handler;

pub struct Openai {
    client: Client<OpenAIConfig>,
    model: String,
}

#[bon]
impl Openai {
    #[builder]
    pub fn new(
        base: impl Into<String>,
        model: impl Into<String>,
        key: impl Into<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Openai> {
        let mut client_builder = reqwest::Client::builder().timeout(timeout);
        if let Some(url) = proxy {
            let proxy = Proxy::all(url)?;
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder.build()?;
        let config = OpenAIConfig::new().with_api_base(base).with_api_key(key);
        let client = Client::with_config(config).with_http_client(client);
        let openai = Openai {
            client,
            model: model.into(),
        };

        Ok(openai)
    }

    pub async fn chat(
        &self,
        content: impl Into<ChatCompletionRequestUserMessageContent>,
    ) -> Result<String> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful assistant.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(content)
                    .build()?
                    .into(),
            ])
            .build()?;
        let response = self.client.chat().create(request).await?;
        let reply = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or(anyhow!("{}没有回复", self.model))?;

        Ok(reply)
    }
}

#[async_trait]
impl Handler for Openai {
    async fn translate(&self, content: &str) -> Result<String> {
        let translated = self
            .chat(formatdoc!(
                "
                请将下面内容翻译为中文，不要输出除了翻译内容外的其他内容

                {content}
                "
            ))
            .await?;

        Ok(translated)
    }
}
