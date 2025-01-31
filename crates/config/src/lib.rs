use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::Deserialize;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct Config {
    #[validate(nested)]
    pub input: Input,
    #[validate(nested)]
    pub output: Output,
}

impl Config {
    pub async fn load() -> Result<Config> {
        let config_path = Config::config_path();
        let config_file = config_path.join("config.toml");
        if !config_file.exists() {
            fs::create_dir_all(config_path).await?;
            Config::generate_default_config_file(&config_file).await?;
            bail!("配置文件不存在, 已生成 > {}", config_file.display());
        }

        let mut config = String::new();
        OpenOptions::new()
            .read(true)
            .open(config_file)
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
            .write_all(include_bytes!("../config.default.toml"))
            .await?;

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

#[derive(Debug, Deserialize, Validate)]
pub struct Input {
    #[validate(custom(function = "absolute_path"))]
    pub path: PathBuf,
    pub exts: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Output {
    #[validate(custom(function = "absolute_path"))]
    pub path: PathBuf,
}

fn absolute_path(path: &Path) -> Result<(), ValidationError> {
    if !path.is_absolute() {
        let msg = format!("路径必须是绝对路径 > {}", path.display());
        let err = ValidationError::new("path").with_message(msg.into());
        return Err(err);
    }

    Ok(())
}
