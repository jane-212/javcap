use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Video {
    #[validate(length(min = 1, message = "至少一个规则"))]
    pub rule: Vec<Tag>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Tag {
    #[serde(rename = "title")]
    Title,

    #[serde(rename = "studio")]
    Studio,

    #[serde(rename = "name")]
    Name,

    #[serde(rename = "id")]
    Id,

    #[serde(rename = "director")]
    Director,

    #[serde(rename = "country")]
    Country,

    #[serde(rename = "actor")]
    Actor,
}
