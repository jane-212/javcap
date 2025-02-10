use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use config::Config;
use log::{error, info, warn};
use tokio::fs;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinSet;
use validator::Validate;
use video::{Video, VideoFile, VideoType};

use super::bar::Bar;
use super::helper::Helper;
use super::message::Message;
use super::payload::Payload;

pub struct App {
    config: Config,
    videos: HashMap<VideoType, Video>,
    tasks: JoinSet<std::result::Result<(), SendError<Message>>>,
    succeed: Vec<String>,
    failed: Vec<String>,
    helper: Arc<Helper>,
    bar: Arc<Bar>,
}

impl App {
    pub async fn new(config: Config) -> Result<App> {
        let helper = Helper::new(&config).with_context(|| "build helper")?;
        let bar = Bar::new().await;
        let app = App {
            tasks: JoinSet::new(),
            config,
            succeed: Vec::new(),
            failed: Vec::new(),
            videos: HashMap::new(),
            helper: Arc::new(helper),
            bar: Arc::new(bar),
        };

        Ok(app)
    }

    async fn start_all_tasks(&mut self) -> Result<Receiver<Message>> {
        let (tx, rx) = mpsc::channel(10);
        for video in self.videos.clone().into_values() {
            let tx = tx.clone();
            let helper = self.helper.clone();
            let bar = self.bar.clone();
            self.tasks.spawn(async move {
                let name = video.ty().name();
                info!("add {name} to queue");
                let msg = match Self::process_video(video, helper, bar).await {
                    Ok(payload) => Message::Loaded(Box::new(payload)),
                    Err(e) => Message::Failed(name, format!("{e:?}")),
                };
                tx.send(msg).await
            });
        }

        Ok(rx)
    }

    pub async fn run(mut self) -> Result<()> {
        self.load_all_videos()
            .await
            .with_context(|| "load videos")?;
        let mut rx = self
            .start_all_tasks()
            .await
            .with_context(|| "start all tasks")?;

        while let Some(msg) = rx.recv().await {
            self.handle_message(msg).await;
        }

        self.wait_for_all_tasks()
            .await
            .with_context(|| "wait for all tasks")?;
        self.summary().await;

        Ok(())
    }

    fn print_bar(&self, name: &str) {
        self.bar.message(format!(
            "{:=^width$}",
            format!(" {} ", name).yellow(),
            width = app::LINE_LENGTH,
        ));
    }

    async fn process_video(video: Video, helper: Arc<Helper>, bar: Arc<Bar>) -> Result<Payload> {
        let _permit = helper
            .sema
            .acquire()
            .await
            .with_context(|| "acquire permit")?;

        let mut nfo = helper
            .spider
            .find(video.ty().clone())
            .await
            .with_context(|| "find video")?;
        nfo.auto_fix();
        info!("{}", nfo.summary());
        nfo.validate().with_context(|| "validate nfo")?;

        let title_task = tokio::spawn({
            let helper = helper.clone();
            let title = nfo.title().clone();
            async move {
                helper
                    .translator
                    .translate(&title)
                    .await
                    .with_context(|| format!("translate {title}"))
            }
        });
        let plot_task = tokio::spawn({
            let helper = helper.clone();
            let plot = nfo.plot().clone();
            async move {
                helper
                    .translator
                    .translate(&plot)
                    .await
                    .with_context(|| format!("translate {plot}"))
            }
        });

        if let Some(title) = title_task.await?? {
            info!("translated {title}");
            nfo.set_title(title);
        }
        if let Some(plot) = plot_task.await?? {
            info!("translated {plot}");
            nfo.set_plot(plot);
        }

        let payload = Payload::builder().video(video).nfo(nfo).bar(bar).build();
        Ok(payload)
    }

    async fn handle_succeed(&mut self, payload: &Payload) -> Result<()> {
        let out = self.get_out_path(payload).await?;
        payload
            .write_all_to(&out)
            .await
            .with_context(|| format!("write payload to {}", out.display()))?;
        payload
            .move_videos_to(&out)
            .await
            .with_context(|| format!("move videos to {}", out.display()))?;

        self.bar.add().await;
        let name = payload.video().ty().name();
        info!("{name} ok");
        self.succeed.push(name);
        Ok(())
    }

    fn concat_rule(&self, payload: &Payload) -> PathBuf {
        let mut out = self.config.output.path.to_path_buf();
        for tag in self.config.output.rule.iter() {
            let name = payload.get_by_tag(tag);
            out = out.join(name);
        }

        out
    }

    async fn get_out_path(&self, payload: &Payload) -> Result<PathBuf> {
        let out = self.concat_rule(payload);
        self.bar.message(format!("to {}", out.display()));
        if out.is_file() {
            bail!("target is a file");
        }

        if !out.exists() {
            fs::create_dir_all(&out)
                .await
                .with_context(|| format!("create dir for {}", out.display()))?;
        }

        Ok(out)
    }

    async fn handle_failed(&mut self, name: String, err: String) {
        self.bar.message(format!("{}", "failed".red()));
        self.bar.message(&err);

        self.bar.add().await;
        error!("{name} failed, caused by {err}");
        self.failed.push(name);
    }

    async fn handle_message(&mut self, msg: Message) {
        self.print_bar(&msg.name());
        match msg {
            Message::Loaded(payload) => {
                if let Err(err) = self.handle_succeed(&payload).await {
                    let name = payload.video().ty().name();
                    self.handle_failed(name, format!("{err:?}")).await;
                }
            }
            Message::Failed(name, err) => {
                self.handle_failed(name, err).await;
            }
        }
    }

    async fn wait_for_all_tasks(&mut self) -> Result<()> {
        while let Some(task) = self.tasks.join_next().await {
            task??;
        }

        Ok(())
    }

    async fn summary(&self) {
        self.bar.finish().await;
        println!(
            "{:=^width$}",
            " Summary ".yellow(),
            width = app::LINE_LENGTH
        );
        info!("ok: {}({})", self.succeed.len(), self.succeed.join(", "));
        println!(
            "{}",
            format!("ok: {}({})", self.succeed.len(), self.succeed.join(", ")).green()
        );
        info!("failed: {}({})", self.failed.len(), self.failed.join(", "));
        println!(
            "{}",
            format!("failed: {}({})", self.failed.len(), self.failed.join(", ")).red()
        );
    }

    async fn load_all_videos(&mut self) -> Result<()> {
        let input = &self.config.input;
        for file in Self::walk_dir(&input.path, &input.excludes)
            .await
            .with_context(|| "walk dir")?
        {
            let name = match file.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let (file_name, ext) = match name.split_once('.') {
                Some(res) => res,
                None => continue,
            };

            if input.exts.iter().any(|e| e == ext) {
                let (video_ty, idx) = match VideoType::parse(file_name) {
                    Ok(res) => res,
                    Err(_) => continue,
                };

                let video = self
                    .videos
                    .entry(video_ty.clone())
                    .or_insert(Video::new(video_ty));
                video.add_file(
                    VideoFile::builder()
                        .location(&file)
                        .ext(ext)
                        .idx(idx)
                        .build(),
                );
            }
        }

        self.bar.set_total(self.videos.len()).await;
        let videos = self
            .videos
            .values()
            .map(|video| video.ty().name())
            .collect::<Vec<_>>()
            .join(", ");
        info!("found videos: {}({})", self.videos.len(), videos);
        self.bar
            .message(format!("found videos: {}({})", self.videos.len(), videos));

        Ok(())
    }

    async fn walk_dir(path: &Path, excludes: &[String]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(path)
            .await
            .with_context(|| format!("read dir in {}", path.display()))?;
        while let Some(entry) = entries.next_entry().await? {
            let file = entry.path();

            let name = match file.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let should_pass = excludes.iter().any(|e| e == name);
            if should_pass {
                warn!("skip {}", file.display());
                continue;
            }

            if file.is_dir() {
                let child_files = Box::pin(Self::walk_dir(&file, excludes)).await?;
                files.extend(child_files);
                continue;
            }

            info!("found video {}", file.display());
            files.push(file);
        }

        Ok(files)
    }
}
