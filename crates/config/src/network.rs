use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Network {
    #[validate(range(min = 1, message = "should be larger than 0"))]
    pub timeout: u64,
    #[validate(url(message = "should be a url"))]
    pub proxy: Option<String>,
}
