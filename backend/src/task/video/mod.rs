use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use config::{Config, Rule};
use engine::{Avsox, Jav321, Javbus, Javdb, Javlib, Mgstage};
use info::Info;
use parser::VideoParser;
use reqwest::Client;
use subtitle::Subtitle;
use tokio::fs;
use tracing::{info, warn};
use translate::Translator;
use walkdir::WalkDir;

use crate::bar::Bar;

use super::Task;

mod engine;
mod info;
pub mod parser;
mod subtitle;
mod translate;

pub struct Video {
    engines: Vec<Arc<Box<dyn Engine>>>,
    translate: Option<Box<dyn Translator>>,
    subtitle: Subtitle,
    remove_empty: bool,
    root: PathBuf,
    exclude: Vec<String>,
    ext: Vec<String>,
    output: String,
    rules: Vec<Rule>,
    other: String,
}

impl Video {
    pub fn new(client: Arc<Client>, config: &Config, pwd: &Path) -> anyhow::Result<Self> {
        let engines: Vec<Arc<Box<dyn Engine>>> = vec![
            Arc::new(Box::new(Javbus::new(client.clone()))),
            Arc::new(Box::new(Javdb::new(client.clone()))),
            Arc::new(Box::new(Javlib::new(client.clone()))),
            Arc::new(Box::new(Jav321::new(client.clone()))),
            Arc::new(Box::new(Avsox::new(client.clone()))),
            Arc::new(Box::new(Mgstage::new(client.clone()))),
        ];
        let translate = match config.video.translate {
            config::Translate::Disable => None,
        };
        let subtitle = Subtitle::new(client);
        let mut root = PathBuf::from(&config.file.root);
        if root.is_relative() {
            root = pwd.join(&root).canonicalize().unwrap_or(root);
        }
        info!("root path {}", root.display());

        Ok(Self {
            engines,
            translate,
            subtitle,
            remove_empty: config.file.remove_empty,
            root,
            exclude: config.file.exclude.clone(),
            ext: config.file.ext.clone(),
            output: config.file.output.clone(),
            rules: config.video.rules.clone(),
            other: config.file.other.clone(),
        })
    }

    pub async fn search(&mut self, video: &VideoParser) -> Info {
        let mut info = Info::new(video.id().to_string());
        let mut handles = Vec::with_capacity(self.engines.len());
        for engine in self.engines.clone() {
            if engine.support(video) {
                info!("search {} in {}", video.id(), engine.id());
                let id = engine.id().to_string();
                let video = video.clone();
                let handle = tokio::spawn(async move { engine.search(&video).await });
                handles.push((id, handle));
            }
        }
        for (id, handle) in handles {
            if let Ok(new_info) = handle.await {
                match new_info {
                    Ok(new_info) => {
                        info!("found {} in {}", video.id(), id);
                        info.merge(new_info);
                    }
                    Err(err) => warn!("{} not found in {id}, caused by {err}", video.id()),
                }
            }
        }

        info
    }

    pub async fn translate(&mut self, info: &mut Info) -> anyhow::Result<()> {
        if let Some(ref translate) = self.translate {
            info!("translate");
            let mut text = BTreeMap::new();
            text.insert("title", info.get_title().to_string());
            text.insert("plot", info.get_plot().to_string());
            let res = translate.translate(text).await;
            if let Some(title) = res.get("title") {
                info.title(title.to_string());
            }
            if let Some(plot) = res.get("plot") {
                info.plot(plot.to_string());
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
                            for exclude in self.exclude.iter() {
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
                path.is_file()
                    && path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| {
                            for e in self.ext.iter() {
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

    async fn handle(&mut self, path: &Path, bar: &mut Bar) -> anyhow::Result<()> {
        match VideoParser::parse(path) {
            Ok(video) => {
                bar.message(video.id());
                let mut info = self.search(&video).await;
                if let Err(err) = self.translate(&mut info).await {
                    warn!("translate {} failed, caused by {err}", video.id());
                }
                self.subtitle.find_subtitle(&mut info).await?;
                let Some(info) = info.check(&video) else {
                    anyhow::bail!("info of {} not complete", video.id());
                };
                info.write_to(
                    &self.root.join(&self.output),
                    path,
                    video.idx(),
                    &self.rules,
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

    async fn remove_empty(&self) -> anyhow::Result<()> {
        let mut entries = fs::read_dir(&self.root).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with('.') {
                    continue;
                }
                for exclude in self.exclude.iter() {
                    if name == exclude {
                        continue;
                    }
                }
                if Self::is_empty(&entry.path()).await? {
                    fs::remove_dir_all(entry.path()).await?;
                    info!("remove {}", entry.path().display());
                }
            }
        }

        Ok(())
    }

    async fn is_empty(path: &Path) -> anyhow::Result<bool> {
        if !path.is_dir() {
            return Ok(false);
        }
        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
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
            let out = self.root.join(&self.other).join(name);
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
}

#[async_trait]
impl Task for Video {
    async fn run(&mut self) -> anyhow::Result<()> {
        let paths = self.walk();
        info!("total {} videos found", paths.len());
        let mut bar = Bar::new(paths.len() as u64)?;
        bar.println("MOVIE");
        for path in paths {
            if let Err(err) = self.handle(&path, &mut bar).await {
                bar.warn(&format!("{}", err));
            }
        }
        if self.remove_empty {
            self.remove_empty().await?;
        }

        Ok(())
    }
}

#[async_trait]
pub trait Engine: Send + Sync {
    async fn search(&self, video: &VideoParser) -> anyhow::Result<Info>;
    fn support(&self, video: &VideoParser) -> bool;
    fn id(&self) -> &'static str;
}

#[macro_export]
macro_rules! select {
    ($($k:ident: $v: expr),*) => {
        struct Selectors {
            $(pub $k: scraper::Selector),*
        }

        impl Selectors {
            fn new() -> Self {
                Self {
                    $($k: scraper::Selector::parse($v).expect(&format!("parse {} failed",stringify!($k)))),*
                }
            }
        }

        fn selectors() -> &'static Selectors {
            static SELECTORS: std::sync::OnceLock<Selectors> = std::sync::OnceLock::new();
            SELECTORS.get_or_init(Selectors::new)
        }
    };
}
