use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Network {
    #[validate(range(min = 1, message = "超时时间必须大于0"))]
    pub timeout: u64,
    #[validate(url(message = "不是url"))]
    pub proxy: Option<String>,
}
