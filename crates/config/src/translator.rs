use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Translator {
    #[serde(rename = "youdao")]
    Youdao { key: String, secret: String },
    #[serde(rename = "deepseek")]
    DeepSeek {
        base: String,
        model: String,
        key: String,
    },
    #[serde(rename = "openai")]
    Openai {
        base: String,
        model: String,
        key: String,
    },
    #[serde(rename = "deepl")]
    DeepL,
}
