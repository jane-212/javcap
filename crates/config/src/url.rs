use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Url {
    #[validate(url(message = "不是url"))]
    pub avsox: Option<String>,

    #[validate(url(message = "不是url"))]
    pub javdb: Option<String>,
}
