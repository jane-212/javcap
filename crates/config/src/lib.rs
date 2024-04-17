use std::path::Path;

use serde::Deserialize;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct Config {
    #[validate(nested)]
    pub app: App,
    #[validate(nested)]
    pub file: File,
    #[validate(nested)]
    pub network: Network,
    #[validate(nested)]
    pub avatar: Avatar,
    #[validate(nested)]
    pub video: Video,
}

#[derive(Deserialize, Validate)]
pub struct App {
    pub quit_on_finish: bool,
}

#[derive(Deserialize, Validate)]
pub struct Video {
    pub translate: bool,
}

#[derive(Deserialize, Validate)]
pub struct Avatar {
    #[validate(url(message = "should be a url"))]
    pub host: String,
    pub api_key: String,
    pub refresh: bool,
}

#[derive(Deserialize, Validate)]
pub struct File {
    #[validate(length(min = 1, message = "should not be emtpy"))]
    pub root: String,
    #[validate(length(min = 1, message = "should not be emtpy"))]
    pub output: String,
    #[validate(length(min = 1, message = "should not be emtpy"))]
    pub other: String,
    pub exclude: Vec<String>,
    pub ext: Vec<String>,
}

#[derive(Deserialize, Validate)]
pub struct Network {
    #[validate(url)]
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
        let mut config: Config =
            toml::from_str(&config).map_err(|err| anyhow::anyhow!("config ->\n\n{err}"))?;
        config
            .validate()
            .map_err(|err| anyhow::anyhow!("config -> {err}"))?;
        config.fix();

        Ok(config)
    }

    fn fix(&mut self) {
        if self.file.output.trim().is_empty() {
            self.file.output = "output".to_string();
        }
        if self.file.other.trim().is_empty() {
            self.file.other = "other".to_string();
        }
        self.file.exclude.push(self.file.output.clone());
        self.file.exclude.push(self.file.other.clone());
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
