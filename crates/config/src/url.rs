use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Url {
    #[validate(url(message = "should be a url"))]
    pub avsox: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub javdb: Option<String>,
}
