use std::fs::OpenOptions;
use std::path::PathBuf;

use anyhow::Result;
use colored::Colorize;
use config::Config;
use env_logger::{Builder, Target};
use javcap::App;
use log::{info, LevelFilter};
use tokio::fs;
use validator::Validate;

#[tokio::main]
async fn main() {
    println!("{}", ">".repeat(app::LINE_LENGTH).yellow());
    if let Err(e) = run().await {
        println!("{:#^width$}", " Error ".red(), width = app::LINE_LENGTH);
        eprintln!("{}", e);
    }
    println!("{}", "<".repeat(app::LINE_LENGTH).yellow());
}

async fn run() -> Result<()> {
    init_logger().await?;

    info!("版本: {}({})", app::VERSION, app::HASH);
    println!("当前版本: {}({})", app::VERSION, app::HASH);

    let config = Config::load().await?;
    config.validate()?;

    let app = App::new(config).await?;

    app.run().await
}

async fn init_logger() -> Result<()> {
    let log_dir = {
        let username = whoami::username();
        #[cfg(target_os = "macos")]
        let user_dir = PathBuf::from("/Users").join(username);
        #[cfg(target_os = "linux")]
        let user_dir = PathBuf::from("/home").join(username);
        #[cfg(target_os = "windows")]
        let user_dir = PathBuf::from("C:\\Users").join(username);

        user_dir.join(".cache").join(app::NAME)
    };
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).await?;
    }
    let log_file = log_dir.join("log");
    if log_file.exists() {
        fs::rename(&log_file, log_dir.join("old.log")).await?;
    }
    let log_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(log_file)?;

    Builder::new()
        .filter_level(LevelFilter::Info)
        .target(Target::Pipe(Box::new(log_file)))
        .init();

    Ok(())
}
