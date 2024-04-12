use std::{
    env,
    path::{Path, PathBuf},
};

use backend::Backend;
use config::Config;
use error::{Result, Error};
use video::Video;
use walkdir::WalkDir;

mod bar;

use bar::Bar;

const CONFIG_NAME: &str = "config.toml";

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        println!("{err}");
    }
}

async fn run() -> Result<()> {
    let pwd = env::current_dir()?;
    let config = Config::load(&pwd.join(CONFIG_NAME)).await?;
    let paths = walk(&pwd, &config);
    let bar = Bar::new(paths.len() as u64)?;
    let backend = Backend::new()?;

    for path in paths {
        if let Err(err) = handle(&path, &bar, &backend).await {
            bar.warn(&err.to_string());
        }
    }

    Ok(())
}

async fn handle(path: &Path, bar: &Bar, backend: &Backend) -> Result<()> {
    let video = Video::parse(path)?;
    bar.message(&format!("search {}", video.id()));
    let Some(info) = backend.search(&video).await else {
        return Err(Error::Info(video.id().to_string()))
    };
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
