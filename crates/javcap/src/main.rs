use std::{
    env,
    path::{Path, PathBuf},
};

use backend::Backend;
use config::Config;
use error::{Error, Result};
use tracing::error;
use tracing_appender::rolling;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use video::Video;
use walkdir::WalkDir;

mod bar;

use bar::Bar;

const CONFIG_NAME: &str = "config.toml";
const LOG_NAME: &str = "logs";

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        error!("{err}");
        println!("{err}");
    }
}

async fn run() -> Result<()> {
    let pwd = env::current_dir()?;
    let file = rolling::daily(pwd.join(LOG_NAME), "info").with_max_level(tracing::Level::INFO);
    tracing_subscriber::fmt()
        .with_writer(file)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .init();
    let config = Config::load(&pwd.join(CONFIG_NAME)).await?;
    let paths = walk(&pwd, &config);
    let bar = Bar::new(paths.len() as u64)?;
    let backend = Backend::new(&config.network.proxy)?;

    for path in paths {
        if let Err(err) = handle(&path, &bar, &backend, &config).await {
            bar.warn(&err.to_string());
        }
    }

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
    bar.info(&format!("{}({})", video.id(), video.path().display()));

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
