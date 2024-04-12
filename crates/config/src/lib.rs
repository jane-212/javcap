use std::path::Path;

use error::{Error, Result};
use serde::Deserialize;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub file: File,
}

#[derive(Deserialize, Debug)]
pub struct File {
    pub output: String,
    pub exclude: Vec<String>,
    pub ext: Vec<String>,
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
