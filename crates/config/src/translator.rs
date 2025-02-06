use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Translator {
    #[serde(rename = "youdao")]
    Youdao { key: String, secret: String },
}
