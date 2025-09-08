use std::fmt::{self, Display};

use anyhow::Result;
use async_trait::async_trait;
use bon::bon;
use deeplx::{Config, DeepLX};

use super::Handler;

pub struct DeepL {
    client: DeepLX,
}

#[bon]
impl DeepL {
    #[builder]
    pub fn new(proxy: Option<String>) -> Result<Self> {
        let client = DeepLX::new(Config {
            proxy,
            ..Default::default()
        });
        let deepl = DeepL { client };

        Ok(deepl)
    }
}

impl Display for DeepL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "deepl")
    }
}

#[async_trait]
impl Handler for DeepL {
    async fn translate(&self, content: &str) -> Result<String> {
        let response = self.client.translate("auto", "zh", content, None).await?;

        Ok(response.data)
    }
}
