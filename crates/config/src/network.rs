use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Network {
    pub timeout: u64,
    #[validate(url)]
    pub proxy: Option<String>,
}
