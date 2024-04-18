use std::{
    env,
    path::{Path, PathBuf},
};

use backend::bar::Bar;
use backend::video::Video;
use backend::Backend;
use config::Config;
use console::style;
use time::{macros::format_description, UtcOffset};
use tokio::fs;
use tracing::{info, warn, Level};
use tracing_appender::rolling;
use tracing_subscriber::fmt::time::OffsetTime;
use walkdir::WalkDir;

pub struct App {
    root: PathBuf,
    config: Config,
    backend: Backend,
}

impl App {
    const CONFIG_NAME: &'static str = "config.toml";
    const LOG_NAME: &'static str = "logs";

    pub async fn new() -> anyhow::Result<Self> {
        let pwd = env::current_dir()?;
        Self::init_tracing(&pwd);
        info!(
            "{:-^30}",
            format!(
                " {} - {} ",
                env!("CARGO_PKG_NAME").to_uppercase(),
                env!("CARGO_PKG_VERSION")
            )
        );
        let config = Config::load(&pwd.join(Self::CONFIG_NAME)).await?;
        info!(
            "config loaded from {}",
            pwd.join(Self::CONFIG_NAME).display()
        );
        let mut root = PathBuf::from(&config.file.root);
        if root.is_relative() {
            root = pwd.join(&root).canonicalize().unwrap_or(root);
        }
        info!("root path {}", root.display());
        let backend = Backend::new(
            &config.network.proxy,
            config.network.timeout,
            &config.avatar.host,
            &config.avatar.api_key,
            &config.video.translate,
        )?;

        Ok(Self {
            root,
            config,
            backend,
        })
    }

    async fn check(&self) -> anyhow::Result<()> {
        let bar = backend::bar::Bar::new_check()?;
        bar.set_message("check network");
        self.backend
            .ping("https://www.javbus.com")
            .await
            .map_err(|err| anyhow::anyhow!("check network failed, caused by {err}"))?;
        bar.finish_and_clear();
        info!("network check passed");
        println!(
            "{:>10} ✔ network check passed",
            style("Check").green().bold()
        );

        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<bool> {
        self.check().await?;
        let paths = self.walk();
        info!("total {} videos found", paths.len());
        {
            let mut bar = Bar::new(paths.len() as u64)?;
            bar.println("MOVIE");
            for path in paths {
                if let Err(err) = self.handle(&path, &mut bar).await {
                    bar.warn(&format!("{}", err));
                }
            }
        }
        if self.config.avatar.refresh {
            self.refresh_avatar().await?;
        }
        if self.config.file.remove_empty {
            self.remove_empty().await?;
        }

        Ok(self.config.app.quit_on_finish)
    }

    async fn refresh_avatar(&self) -> anyhow::Result<()> {
        self.backend.refresh_avatar().await
    }

    async fn remove_empty(&self) -> anyhow::Result<()> {
        let mut entrys = fs::read_dir(&self.root).await?;
        while let Some(entry) = entrys.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with('.') {
                    continue;
                }
                for exclude in self.config.file.exclude.iter() {
                    if name == exclude {
                        continue;
                    }
                }
                if App::is_empty(&entry.path()).await? {
                    fs::remove_dir_all(entry.path()).await?;
                    info!("remove {}", entry.path().display());
                }
            }
        }

        Ok(())
    }

    async fn is_empty(path: &Path) -> anyhow::Result<bool> {
        let mut entrys = fs::read_dir(path).await?;
        while let Some(entry) = entrys.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn move_to_other(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(name) = path.file_stem().and_then(|name| name.to_str()) {
            let ext = path.extension().and_then(|ext| ext.to_str());
            let to_file = match ext {
                Some(ext) => format!("{}.{}", name, ext),
                None => name.to_string(),
            };
            let out = self.root.join(&self.config.file.other).join(name);
            let out_file = out.join(&to_file);
            if out_file.exists() {
                anyhow::bail!("video {} already exists", out_file.display());
            }
            fs::create_dir_all(&out).await?;
            info!("create {}", out.display());
            fs::rename(path, &out_file).await?;
            info!("move {} to {}", path.display(), out_file.display());
        }

        Ok(())
    }

    async fn handle(&mut self, path: &Path, bar: &mut Bar) -> anyhow::Result<()> {
        match Video::parse(path) {
            Ok(video) => {
                bar.message(&format!("search {}", video.id()));
                let Some(mut info) = self.backend.search(&video).await else {
                    anyhow::bail!("info of {} not complete", video.id());
                };
                if let Err(err) = self.backend.translate(&mut info).await {
                    warn!("translate {} failed, caused by {err}", video.id());
                }
                bar.message(&format!("write {}", video.id()));
                info.write_to(
                    &self.root.join(&self.config.file.output),
                    path,
                    video.idx(),
                    &self.config.video.rules,
                )
                .await
                .map_err(|err| anyhow::anyhow!("save info failed, caused by {err}"))?;
                bar.info(video.id());
            }
            Err(err) => {
                self.move_to_other(path).await.map_err(|err| {
                    anyhow::anyhow!("move video to other failed, caused by {err}")
                })?;
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
        let daily = rolling::daily(path.join(Self::LOG_NAME), "log");
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
