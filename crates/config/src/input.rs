use std::path::PathBuf;

use serde::Deserialize;
use validator::Validate;

use super::helper::absolute_path;

#[derive(Debug, Deserialize, Validate)]
pub struct Input {
    #[validate(custom(function = "absolute_path"))]
    pub path: PathBuf,
    pub exts: Vec<String>,
    pub excludes: Vec<String>,
}
