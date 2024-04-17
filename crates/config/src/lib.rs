use std::path::Path;

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
    pub avatar: Avatar,
}

#[derive(Deserialize)]
pub struct App {
    pub quit_on_finish: bool,
}

#[derive(Deserialize)]
pub struct Avatar {
    pub host: String,
    pub api_key: String,
    pub refresh: bool,
}

#[derive(Deserialize)]
pub struct File {
    pub root: String,
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
    pub async fn load(path: &Path) -> anyhow::Result<Config> {
        if !path.exists() {
            Config::generate_default_config(path).await?;

            anyhow::bail!("config file not found, auto generate");
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

    async fn generate_default_config(path: &Path) -> anyhow::Result<()> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .await?
            .write_all(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/config.default.toml"
            )))
            .await?;

        Ok(())
    }
}
