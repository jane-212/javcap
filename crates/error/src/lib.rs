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
    #[error("id not found({0})")]
    Parse(String),
    #[error("network error")]
    Client(#[from] reqwest::Error),
    #[error("info not complete({0})")]
    Info(String),
    #[error("movie already exists({0})")]
    AlreadyExists(String),
}

pub type Result<T> = std::result::Result<T, Error>;
