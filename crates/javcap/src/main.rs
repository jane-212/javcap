use std::env;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use colored::Colorize;
use config::Config;
use env_logger::{Builder, Target};
use javcap::App;
use log::info;
use self_update::backends::github::Update;
use self_update::Status;
use tokio::fs;
use validator::Validate;

#[tokio::main]
async fn main() -> ExitCode {
    println!("{}", ">".repeat(app::LINE_LENGTH).yellow());
    let code = match run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{:#^width$}", " Error ".red(), width = app::LINE_LENGTH);
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    };
    println!("{}", "<".repeat(app::LINE_LENGTH).yellow());
    code
}

async fn run() -> Result<()> {
    init_logger().await?;

    info!("版本: v{}({})", app::VERSION, app::HASH);
    println!("当前版本: v{}({})", app::VERSION, app::HASH);

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
        println!("\n已是最新版本");
    }

    let app = App::new(config).await?;

    app.run().await
}

fn check_for_update() -> Result<Status> {
    let status = Update::configure()
        .repo_owner("jane-212")
        .repo_name("javcap")
        .bin_name("javcap")
        .bin_path_in_archive("javcap-{{version}}-{{target}}/{{bin}}")
        .no_confirm(true)
        .show_output(false)
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

    if env::var("LOG")
        .ok()
        .map(|log| log.is_empty())
        .unwrap_or(true)
    {
        env::set_var("LOG", "info");
    }
    Builder::from_env("LOG")
        .target(Target::Pipe(Box::new(log_file)))
        .init();

    Ok(())
}
