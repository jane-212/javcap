use std::path::Path;

use error::{Error, Result};
use serde::Deserialize;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Deserialize)]
pub struct Config {
    pub app: App,
    pub file: File,
    pub network: Network,
}

#[derive(Deserialize)]
pub struct App {
    pub quit_on_finish: bool,
}

#[derive(Deserialize)]
pub struct File {
    pub root: String,
    pub remove_empty: bool,
    pub output: String,
    pub other: String,
    pub exclude: Vec<String>,
    pub ext: Vec<String>,
}

#[derive(Deserialize)]
pub struct Network {
    pub proxy: String,
    pub timeout: u64,
}

impl Config {
    pub async fn load(path: &Path) -> Result<Config> {
        if !path.exists() {
            Config::generate_default_config(path).await?;

            return Err(Error::ConfigNotFound {
                path: path.to_path_buf(),
            });
        }
        let mut config = String::new();
        OpenOptions::new()
            .read(true)
            .open(path)
            .await?
            .read_to_string(&mut config)
            .await?;
        let mut config: Config = toml::from_str(&config)?;
        config.file.exclude.push(config.file.output.clone());
        config.file.exclude.push(config.file.other.clone());

        Ok(config)
    }

    async fn generate_default_config(path: &Path) -> Result<()> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .await?
            .write_all(
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/config.default.toml"))
                    .as_bytes(),
            )
            .await?;

        Ok(())
    }
}
