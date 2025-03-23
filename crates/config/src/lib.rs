mod helper;
mod input;
mod network;
mod output;
mod the_porn_db;
mod translator;
mod url;

pub use output::Tag;
pub use translator::Translator;

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use input::Input;
use log::info;
use network::Network;
use output::Output;
use serde::Deserialize;
use the_porn_db::ThePornDB;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::Url;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Config {
    pub check_for_update: bool,

    #[validate(range(min = 1, message = "should be larger than 0"))]
    pub task_limit: usize,

    pub translators: Option<Vec<Translator>>,

    #[validate(nested)]
    pub input: Input,

    #[validate(nested)]
    pub output: Output,

    #[validate(nested)]
    pub network: Network,

    #[validate(nested)]
    pub url: Url,

    pub the_porn_db: ThePornDB,
}

impl Config {
    pub const DEFAULT_CONFIG: &str = include_str!("../config.default.toml");

    pub async fn load() -> Result<Config> {
        let config_path = Config::config_path();
        let config_file = config_path.join("config.toml");
        if !config_file.exists() {
            info!("config not found in {}", config_file.display());
            fs::create_dir_all(config_path).await?;
            Config::generate_default_config_file(&config_file).await?;
            bail!(
                "config not found, default config generated to {}",
                config_file.display()
            );
        }

        Self::load_and_decode(config_file).await
    }

    pub async fn load_from(path: impl AsRef<Path>) -> Result<Config> {
        let config_file = path.as_ref();
        if !config_file.exists() {
            info!("config not found in {}", config_file.display());
            bail!("config not found in {}", config_file.display());
        }

        Self::load_and_decode(config_file).await
    }

    async fn load_and_decode(path: impl AsRef<Path>) -> Result<Config> {
        let path = path.as_ref();
        info!("load config from {}", path.display());

        let mut config = String::new();
        OpenOptions::new()
            .read(true)
            .open(path)
            .await?
            .read_to_string(&mut config)
            .await?;
        let config = toml::from_str::<Config>(&config)?;

        Ok(config)
    }

    async fn generate_default_config_file(path: &Path) -> Result<()> {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .await?
            .write_all(Self::DEFAULT_CONFIG.as_bytes())
            .await?;
        info!("generate default config to {}", path.display());

        Ok(())
    }

    /// macos -> /Users/<username>/.config/javcap
    /// linux -> /home/<username>/.config/javcap
    /// windows -> C:\Users\<username>\.config\javcap
    fn config_path() -> PathBuf {
        let username = whoami::username();
        #[cfg(target_os = "macos")]
        let user_dir = PathBuf::from("/Users").join(username);
        #[cfg(target_os = "linux")]
        let user_dir = PathBuf::from("/home").join(username);
        #[cfg(target_os = "windows")]
        let user_dir = PathBuf::from("C:\\Users").join(username);

        user_dir.join(".config").join(app::NAME)
    }
}
