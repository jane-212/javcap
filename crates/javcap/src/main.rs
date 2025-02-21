use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use config::Config;
use env_logger::{Builder, Target};
use javcap::App;
use log::{LevelFilter, error, info};
use self_update::Status;
use self_update::backends::github::Update;
use tokio::fs;
use validator::Validate;

#[tokio::main]
async fn main() -> ExitCode {
    println!("{}", ">".repeat(app::LINE_LENGTH).yellow());
    let code = match run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{:#^width$}", " Error ".red(), width = app::LINE_LENGTH);
            eprintln!("{}", format!("{e:?}").red());
            error!("{e:?}");
            ExitCode::FAILURE
        }
    };
    println!("{}", "<".repeat(app::LINE_LENGTH).yellow());
    code
}

async fn run() -> Result<()> {
    init_logger().await.with_context(|| "init logger")?;

    info!("app version: v{}({})", app::VERSION, app::HASH);
    println!("app version: v{}({})", app::VERSION, app::HASH);

    let config = Config::load().await.with_context(|| "load config")?;
    config.validate().with_context(|| "validate config")?;

    if config.check_for_update {
        info!("check for update...");
        println!("check for update...");
        let status = tokio::task::spawn_blocking(check_for_update).await??;
        if status.updated() {
            info!("updated to version v{}", status.version());
            println!("updated to version v{}", status.version());
            return Ok(());
        }

        info!("latest version, skip");
        println!("latest version, skip");
    }

    let app = App::new(config).await.with_context(|| "init app")?;

    app.run().await.with_context(|| "run app")
}

fn check_for_update() -> Result<Status> {
    let status = Update::configure()
        .repo_owner("jane-212")
        .repo_name("javcap")
        .bin_name("javcap")
        .no_confirm(true)
        .show_output(false)
        .show_download_progress(true)
        .current_version(app::VERSION)
        .build()
        .with_context(|| "build update config")?
        .update()
        .with_context(|| "self update")?;

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
        fs::create_dir_all(&log_dir)
            .await
            .with_context(|| format!("create dir {}", log_dir.display()))?;
    }
    let log_file = log_dir.join("log");
    if log_file.exists() {
        let to = log_dir.join("old.log");
        fs::rename(&log_file, &to)
            .await
            .with_context(|| format!("rename {} to {}", log_file.display(), to.display()))?;
    }
    let log_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&log_file)
        .with_context(|| format!("open {}", log_file.display()))?;

    const LOG_ENV_KEY: &str = "LOG";
    let mut logger = if env::var(LOG_ENV_KEY)
        .map(|log| log.is_empty())
        .unwrap_or(true)
    {
        // fallback to info level if `LOG` is not set or empty
        let mut logger = Builder::new();
        logger.filter_level(LevelFilter::Info);
        logger
    } else {
        Builder::from_env(LOG_ENV_KEY)
    };
    logger
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {:<5} {}] {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.target(),
                record.args(),
            )
        })
        .target(Target::Pipe(Box::new(log_file)))
        .init();

    Ok(())
}
