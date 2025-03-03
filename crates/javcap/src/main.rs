use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use colored::Colorize;
use config::Config;
use env_logger::{Builder, Target};
use javcap::App;
use log::{LevelFilter, error, info};
use self_update::Status;
use self_update::backends::github::Update;
use tokio::fs;
use validator::Validate;

#[derive(Parser)]
#[command(version = app::VERSION)]
#[command(long_about = "电影刮削器")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 搜索并刮削
    Run {
        /// 配置文件路径
        #[arg(short, long)]
        config: Option<String>,
    },

    /// 显示默认配置
    Config,

    /// 显示上次运行的日志
    Log,

    /// 更新程序
    Upgrade,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(command) => match command {
            Commands::Run { config } => run(config).await,
            Commands::Config => {
                println!("{}", Config::DEFAULT_CONFIG.trim_end());
                ExitCode::SUCCESS
            }
            Commands::Log => log().await,
            Commands::Upgrade => upgrade().await,
        },
        None => run(None).await,
    }
}

async fn upgrade() -> ExitCode {
    match _upgrade().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e:?}");
            ExitCode::FAILURE
        }
    }
}

async fn _upgrade() -> Result<()> {
    println!("check for update...");
    let status = tokio::task::spawn_blocking(check_for_update).await??;
    if status.updated() {
        println!("updated to version v{}", status.version());
    } else {
        println!("latest version, nothing to do today");
    }

    Ok(())
}

async fn log() -> ExitCode {
    let log_dir = log_dir();
    let log_file = log_dir.join("log");

    if !log_file.exists() {
        eprintln!("no log file found");
        return ExitCode::FAILURE;
    }

    match fs::read_to_string(&log_file)
        .await
        .with_context(|| "read log file")
    {
        Ok(content) => {
            println!("{}", content.trim_end());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e:?}");
            ExitCode::FAILURE
        }
    }
}

async fn run(config: Option<String>) -> ExitCode {
    println!("{}", ">".repeat(app::LINE_LENGTH).yellow());
    let code = match _run(config).await {
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

async fn _run(config: Option<String>) -> Result<()> {
    init_logger().await.with_context(|| "init logger")?;

    info!("app version: v{}({})", app::VERSION, app::HASH);
    println!("app version: v{}({})", app::VERSION, app::HASH);

    let config = match config {
        Some(path) => Config::load_from(path)
            .await
            .with_context(|| "load config")?,
        None => Config::load().await.with_context(|| "load config")?,
    };
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

fn log_dir() -> PathBuf {
    let username = whoami::username();
    #[cfg(target_os = "macos")]
    let user_dir = PathBuf::from("/Users").join(username);
    #[cfg(target_os = "linux")]
    let user_dir = PathBuf::from("/home").join(username);
    #[cfg(target_os = "windows")]
    let user_dir = PathBuf::from("C:\\Users").join(username);

    user_dir.join(".cache").join(app::NAME)
}

async fn init_logger() -> Result<()> {
    let log_dir = log_dir();
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
