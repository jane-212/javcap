use std::path::PathBuf;

use serde::Deserialize;
use validator::Validate;

use super::helper::absolute_path;

#[derive(Debug, Deserialize, Validate)]
pub struct Output {
    #[validate(custom(function = "absolute_path"))]
    pub path: PathBuf,
}
