use std::{
    env,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use backend::{Backend, Video};
use config::Config;
use console::style;
use error::{Error, Result};
use time::{macros::format_description, UtcOffset};
use tracing::{error, info, Level};
use tracing_appender::rolling;
use tracing_subscriber::fmt::time::OffsetTime;
use walkdir::WalkDir;

mod bar;

use bar::Bar;

const CONFIG_NAME: &str = "config.toml";
const LOG_NAME: &str = "logs";

#[tokio::main]
async fn main() {
    match run().await {
        Ok(should_quit) => {
            info!("{:-^30}", " Finish ");
            if !should_quit {
                wait_for_quit();
            }
        }
        Err(err) => {
            error!("{err}");
            info!("{:-^30}", " Finish ");
            println!("{:>10} {}", style("Error").red().bold(), err);
            wait_for_quit();
        }
    }
}

fn wait_for_quit() {
    print!(
        "{:>10} Press enter to continue...",
        style("Pause").green().bold()
    );
    io::stdout().flush().ok();
    io::stdin().read_exact(&mut [0u8]).ok();
}

async fn run() -> Result<bool> {
    let pwd = env::current_dir()?;
    init_tracing(&pwd)?;
    info!(
        "{:-^30}",
        format!(
            " {} - {} ",
            env!("CARGO_PKG_NAME").to_uppercase(),
            env!("CARGO_PKG_VERSION")
        )
    );
    let config = Config::load(&pwd.join(CONFIG_NAME)).await?;
    info!("config loaded");
    let paths = walk(&pwd, &config);
    info!("total {} videos found", paths.len());
    let bar = Bar::new(paths.len() as u64)?;
    let backend = Backend::new(&config.network.proxy, config.network.timeout)?;
    for path in paths {
        if let Err(err) = handle(&path, &bar, &backend, &config).await {
            bar.warn(&format!("{}({})", err, path.display()));
        }
    }

    Ok(config.app.quit_on_finish)
}

fn init_tracing(path: &Path) -> Result<()> {
    let daily = rolling::daily(path.join(LOG_NAME), "log");
    let timer = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).expect("set timezone error"),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    );
    tracing_subscriber::fmt()
        .with_writer(daily)
        .with_max_level(Level::INFO)
        .with_ansi(false)
        .with_target(false)
        .with_timer(timer)
        .init();

    Ok(())
}

async fn handle(path: &Path, bar: &Bar, backend: &Backend, config: &Config) -> Result<()> {
    let video = Video::parse(path)?;
    bar.message(&format!("search {}", video.id()));
    let Some(info) = backend.search(&video).await else {
        return Err(Error::Info(video.id().to_string()));
    };
    bar.message(&format!("write {}", video.id()));
    info.write_to(&PathBuf::from(&config.file.output), path)
        .await?;
    bar.info(&format!("{}({})", video.id(), path.display()));

    Ok(())
}

fn walk(path: &Path, config: &Config) -> Vec<PathBuf> {
    WalkDir::new(path)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|entry| {
                    !entry.starts_with('.') && {
                        for exclude in config.file.exclude.iter() {
                            if entry == exclude {
                                return false;
                            }
                        }

                        true
                    }
                })
                .unwrap_or(false)
        })
        .flat_map(|entry| entry.ok())
        .map(|entry| entry.into_path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    for e in config.file.ext.iter() {
                        if ext == e {
                            return true;
                        }
                    }

                    false
                })
                .unwrap_or(false)
        })
        .collect()
}
