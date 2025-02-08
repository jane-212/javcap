use std::fs::OpenOptions;
use std::path::PathBuf;

use anyhow::Result;
use colored::Colorize;
use config::Config;
use env_logger::{Builder, Target};
use javcap::App;
use log::{info, LevelFilter};
use self_update::backends::github::Update;
use self_update::Status;
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

    if config.check_for_update {
        info!("正在检查更新...");
        println!("正在检查更新...");
        let status = tokio::task::spawn_blocking(check_for_update).await??;
        if status.updated() {
            info!("已更新为版本: v{}", status.version());
            println!("已更新为版本: v{}", status.version());
            return Ok(());
        }

        info!("已是最新版本");
        println!("已是最新版本");
    }

    let app = App::new(config).await?;

    app.run().await
}

fn check_for_update() -> Result<Status> {
    let status = Update::configure()
        .repo_owner("jane-212")
        .repo_name("javcap")
        .bin_name("javcap")
        .show_download_progress(true)
        .current_version(app::VERSION)
        .build()?
        .update()?;

    Ok(status)
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
