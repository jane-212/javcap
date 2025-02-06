use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Result};
use colored::Colorize;
use config::Config;
use tokio::fs;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinSet;
use validator::Validate;
use video::{Video, VideoFile, VideoType};

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
}

impl App {
    pub fn new(config: Config) -> Result<App> {
        let helper = Helper::new(&config)?;
        let app = App {
            tasks: JoinSet::new(),
            config,
            succeed: Vec::new(),
            failed: Vec::new(),
            videos: HashMap::new(),
            helper: Arc::new(helper),
        };

        Ok(app)
    }

    fn has_finished(&self) -> usize {
        self.succeed.len() + self.failed.len()
    }

    async fn start_all_tasks(&mut self) -> Result<Receiver<Message>> {
        let (tx, rx) = mpsc::channel(10);
        for video in self.videos.clone().into_values() {
            let tx = tx.clone();
            let helper = self.helper.clone();
            self.tasks.spawn(async move {
                let name = video.ty().name();
                let msg = match Self::process_video(video, helper).await {
                    Ok(payload) => Message::Loaded(Box::new(payload)),
                    Err(e) => Message::Failed(name, e.to_string()),
                };
                tx.send(msg).await
            });
        }

        Ok(rx)
    }

    pub async fn run(mut self) -> Result<()> {
        self.load_all_videos().await?;
        let mut rx = self.start_all_tasks().await?;

        while let Some(msg) = rx.recv().await {
            self.handle_message(msg).await?;
        }

        self.wait_for_all_tasks().await?;
        self.summary();

        Ok(())
    }

    fn print_bar(&self, name: &str) {
        println!(
            "{:=^width$}",
            format!(
                " {} ({}/{}) ",
                name,
                self.has_finished() + 1,
                self.videos.len(),
            )
            .yellow(),
            width = app::LINE_LENGTH,
        );
    }

    async fn process_video(video: Video, helper: Arc<Helper>) -> Result<Payload> {
        let _permit = helper.sema.acquire().await?;

        let mut nfo = helper.spider.find(video.ty().clone()).await?;
        nfo.validate()?;

        let title_task = tokio::spawn({
            let helper = helper.clone();
            let title = nfo.title().clone();
            async move { helper.translator.translate(&title).await }
        });
        let plot_task = tokio::spawn({
            let helper = helper.clone();
            let plot = nfo.plot().clone();
            async move { helper.translator.translate(&plot).await }
        });

        if let Some(title) = title_task.await?? {
            nfo.set_title(title);
        }
        if let Some(plot) = plot_task.await?? {
            nfo.set_plot(plot);
        }

        Ok(Payload::new(video, nfo))
    }

    async fn handle_succeed(&mut self, payload: &Payload) -> Result<()> {
        let out = self.get_out_path(payload).await?;
        payload.write_all_to(&out).await?;
        payload.move_videos_to(&out).await?;

        println!("{}", "ok".green());
        let name = payload.video().ty().name();
        self.succeed.push(name);
        Ok(())
    }

    fn concat_rule(&self, payload: &Payload) -> PathBuf {
        let mut out = self.config.output.path.to_path_buf();
        for tag in self.config.video.rule.iter() {
            let name = payload.get_by_tag(tag);
            out = out.join(name);
        }

        out
    }

    async fn get_out_path(&self, payload: &Payload) -> Result<PathBuf> {
        let out = self.concat_rule(payload);
        println!("输出路径 > {}", out.display());
        if out.is_file() {
            bail!("输出路径是文件, 无法创建文件夹");
        }

        if !out.exists() {
            fs::create_dir_all(&out).await?;
        }

        Ok(out)
    }

    fn handle_failed(&mut self, name: String, err: String) {
        println!("{}", "failed".red());
        println!("{err}");

        self.failed.push(name);
    }

    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        self.print_bar(&msg.name());
        match msg {
            Message::Loaded(payload) => {
                if let Err(err) = self.handle_succeed(&payload).await {
                    let name = payload.video().ty().name();
                    self.handle_failed(name, err.to_string());
                }
            }
            Message::Failed(name, err) => {
                self.handle_failed(name, err);
            }
        }

        Ok(())
    }

    async fn wait_for_all_tasks(&mut self) -> Result<()> {
        while let Some(task) = self.tasks.join_next().await {
            task??;
        }

        Ok(())
    }

    fn summary(&self) {
        println!(
            "{:=^width$}",
            " Summary ".yellow(),
            width = app::LINE_LENGTH
        );
        println!(
            "{}",
            format!("成功: {}({})", self.succeed.len(), self.succeed.join(", ")).green()
        );
        println!(
            "{}",
            format!("失败: {}({})", self.failed.len(), self.failed.join(", ")).red()
        );
    }

    async fn load_all_videos(&mut self) -> Result<()> {
        let input = &self.config.input;
        for file in Self::walk_dir(&input.path, &input.excludes).await? {
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
                video.add_file(VideoFile::new(&file, ext, idx));
            }
        }

        println!(
            "共找到视频: {}({})",
            self.videos.len(),
            self.videos
                .values()
                .map(|video| video.ty().name())
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(())
    }

    async fn walk_dir(path: &Path, excludes: &[String]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut entrys = fs::read_dir(path).await?;
        while let Some(entry) = entrys.next_entry().await? {
            let file = entry.path();

            let name = match file.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let should_pass = excludes.iter().any(|e| e == name);
            if should_pass {
                continue;
            }

            if file.is_dir() {
                let child_files = Box::pin(Self::walk_dir(&file, excludes)).await?;
                files.extend(child_files);
                continue;
            }

            files.push(file);
        }

        Ok(files)
    }
}
