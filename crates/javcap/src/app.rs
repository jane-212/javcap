use std::{
    env,
    path::{Path, PathBuf},
    time::Duration,
};

use backend::video::Video;
use backend::Backend;
use config::Config;
use error::{Error, Result};
use indicatif::{ProgressBar, ProgressStyle};
use time::{macros::format_description, UtcOffset};
use tokio::fs;
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::fmt::time::OffsetTime;
use walkdir::WalkDir;

use crate::bar::Bar;

pub struct App {
    root: PathBuf,
    config: Config,
    backend: Backend,
}

impl App {
    const CONFIG_NAME: &'static str = "config.toml";
    const LOG_NAME: &'static str = "logs";

    pub async fn new() -> Result<App> {
        let pwd = env::current_dir()?;
        App::init_tracing(&pwd);
        info!(
            "{:-^30}",
            format!(
                " {} - {} ",
                env!("CARGO_PKG_NAME").to_uppercase(),
                env!("CARGO_PKG_VERSION")
            )
        );
        let config = Config::load(&pwd.join(App::CONFIG_NAME)).await?;
        info!("config loaded");
        let mut root = PathBuf::from(&config.file.root);
        if root.is_relative() {
            root = pwd.join(&root).canonicalize().unwrap_or(root);
        }
        info!("root {}", root.display());
        let backend = Backend::new(&config.network.proxy, config.network.timeout)?;
        let network_bar = ProgressBar::new_spinner();
        network_bar.enable_steady_tick(Duration::from_millis(100));
        network_bar.set_style(
            ProgressStyle::with_template("{prefix:>10.blue.bold} {spinner} {msg}")?
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );
        network_bar.set_prefix("Check");
        network_bar.set_message("checking network");
        backend.ping("https://www.javbus.com").await?;
        network_bar.finish_and_clear();
        info!("network check passed");

        Ok(App {
            root,
            config,
            backend,
        })
    }

    pub async fn run(&mut self) -> Result<bool> {
        let paths = self.walk();
        info!("total {} videos found", paths.len());
        let mut bar = Bar::new(paths.len() as u64)?;
        for path in paths {
            if let Err(err) = self.handle(&path, &mut bar).await {
                bar.warn(&format!("{}", err));
            }
        }

        Ok(self.config.app.quit_on_finish)
    }

    async fn move_to_other(&self, path: &Path) -> Result<()> {
        if let Some(name) = path.file_stem().and_then(|name| name.to_str()) {
            let ext = path.extension().and_then(|ext| ext.to_str());
            let to_file = match ext {
                Some(ext) => format!("{}.{}", name, ext),
                None => name.to_string(),
            };
            let out = self.root.join(&self.config.file.other).join(name);
            let out_file = out.join(&to_file);
            if out_file.exists() {
                return Err(Error::AlreadyExists(out_file.display().to_string()));
            }
            fs::create_dir_all(&out).await?;
            info!("create {}", out.display());
            fs::rename(path, &out_file).await?;
            info!("move {} to {}", path.display(), out_file.display());
        }

        Ok(())
    }

    async fn handle(&mut self, path: &Path, bar: &mut Bar) -> Result<()> {
        match Video::parse(path) {
            Ok(video) => {
                bar.message(&format!("search {}", video.id()));
                let Some(info) = self.backend.search(&video).await else {
                    return Err(Error::Info(video.id().to_string()));
                };
                bar.message(&format!("write {}", video.id()));
                info.write_to(&self.root.join(&self.config.file.output), path)
                    .await?;
                bar.info(video.id());
            }
            Err(err) => {
                self.move_to_other(path).await?;
                return Err(err);
            }
        }

        Ok(())
    }

    fn walk(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.root)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|entry| {
                        !entry.starts_with('.') && {
                            for exclude in self.config.file.exclude.iter() {
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
                        for e in self.config.file.ext.iter() {
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

    fn init_tracing(path: &Path) {
        let daily = rolling::daily(path.join(App::LOG_NAME), "log");
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
    }
}
