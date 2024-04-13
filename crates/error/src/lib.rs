use std::path::PathBuf;

use indicatif::style;
use thiserror::Error;
use tokio::io;
use toml::de;

#[derive(Error, Debug)]
pub enum Error {
    #[error("config not found in {}, auto generate for you", path.display())]
    ConfigNotFound { path: PathBuf },
    #[error("write or read file error")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Config(#[from] de::Error),
    #[error("template error")]
    Template(#[from] style::TemplateError),
    #[error("id not found: {}", path.display())]
    Parse { path: PathBuf },
    #[error("network error")]
    Client(#[from] reqwest::Error),
    #[error("info of {0} not complete")]
    Info(String),
    #[error("movie {0} already exists")]
    AlreadyExists(String),
}

pub type Result<T> = std::result::Result<T, Error>;
