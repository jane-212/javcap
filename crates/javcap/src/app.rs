use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use config::Config;
use tokio::fs;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinSet;
use video::{Video, VideoFile, VideoType};

use super::message::Message;

pub struct App {
    config: Config,
    videos: HashMap<VideoType, Video>,
    tasks: JoinSet<std::result::Result<(), SendError<Message>>>,
    succeed: Vec<String>,
    failed: Vec<String>,
}

impl App {
    pub fn new(config: Config) -> App {
        App {
            tasks: JoinSet::new(),
            config,
            succeed: Vec::new(),
            failed: Vec::new(),
            videos: HashMap::new(),
        }
    }

    fn has_finished(&self) -> usize {
        self.succeed.len() + self.failed.len()
    }

    async fn start_all_tasks(&mut self) -> Result<Receiver<Message>> {
        let (tx, rx) = mpsc::channel(10);
        for video in self.videos.clone().into_values() {
            let tx = tx.clone();
            self.tasks.spawn(async move {
                let name = video.ty().name();
                let msg = match Self::process_video(video).await {
                    Ok(video) => Message::Load(Box::new(video)),
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

    fn print_progress(&self, name: &str) {
        println!(
            "{:=^width$}",
            format!(" {}({}/{}) ", name, self.has_finished(), self.videos.len()),
            width = app::LINE_LENGTH
        );
    }

    async fn handle_succeed(&mut self, video: Box<Video>) -> Result<()> {
        let name = video.ty().name();
        self.succeed.push(name.clone());
        self.print_progress(&name);
        println!("{:#?}", video);

        Ok(())
    }

    fn handle_failed(&mut self, name: String, err: String) {
        self.failed.push(name.clone());
        self.print_progress(&name);
        println!("失败了");
        println!("{err}")
    }

    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Load(video) => {
                self.handle_succeed(video).await?;
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
        println!("{:=^width$}", " Summary ", width = app::LINE_LENGTH);
        println!("成功: {}({})", self.succeed.len(), self.succeed.join(", "));
        println!("失败: {}({})", self.failed.len(), self.failed.join(", "));
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

    async fn process_video(video: Video) -> Result<Video> {
        match video.ty() {
            VideoType::Jav(_, _) => {}
            VideoType::Fc2(_) => anyhow::bail!("fc2 error"),
        }

        Ok(video)
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
