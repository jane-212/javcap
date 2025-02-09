use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use bon::bon;
use colored::Colorize;
use config::Tag;
use getset::Getters;
use log::info;
use nfo::Nfo;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use video::Video;

use super::bar::Bar;

#[derive(Getters)]
pub struct Payload {
    #[getset(get = "pub")]
    video: Video,
    nfo: Nfo,
    bar: Arc<Bar>,
}

#[bon]
impl Payload {
    #[builder]
    pub fn new(video: Video, nfo: Nfo, bar: Arc<Bar>) -> Payload {
        Payload { video, nfo, bar }
    }

    async fn write_fanart_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty().name();
        let filename = format!("{name}-fanart.jpg");
        let file = path.join(filename);
        Self::write_to_file(self.nfo.fanart(), &file).await?;
        info!("背景({}) > {}", name, file.display());
        self.bar.message(format!("背景...{}", "ok".green()));

        Ok(())
    }

    async fn write_poster_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty().name();
        let filename = format!("{name}-poster.jpg");
        let file = path.join(filename);
        Self::write_to_file(self.nfo.poster(), &file).await?;
        info!("封面({}) > {}", name, file.display());
        self.bar.message(format!("封面...{}", "ok".green()));

        Ok(())
    }

    async fn write_to_file(bytes: &[u8], file: &Path) -> Result<()> {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file)
            .await?
            .write_all(bytes)
            .await?;

        Ok(())
    }

    async fn write_nfo_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty().name();
        let filename = format!("{name}.nfo");
        let file = path.join(filename);
        let nfo = self.nfo.to_string();
        Self::write_to_file(nfo.as_bytes(), &file).await?;
        info!("nfo({}) > {}", name, file.display());
        self.bar.message(format!("nfo...{}", "ok".green()));

        Ok(())
    }

    async fn write_subtitle_to(&self, path: &Path) -> Result<()> {
        if self.nfo.subtitle().is_empty() {
            return Ok(());
        }

        let name = self.video.ty().name();
        let filename = format!("{name}.srt");
        let file = path.join(filename);
        Self::write_to_file(self.nfo.subtitle(), &file).await?;
        info!("字幕({}) > {}", name, file.display());
        self.bar.message(format!("字幕...{}", "ok".green()));

        Ok(())
    }

    pub async fn move_videos_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty().name();
        for video in self.video.files() {
            let idx = video.idx();
            let filename = if *idx == 0 {
                format!("{}.{}", name, video.ext())
            } else {
                format!("{}-{}.{}", name, idx, video.ext())
            };
            let out = path.join(&filename);
            if out.exists() {
                info!("文件已存在 > {}", out.display());
                self.bar.message(format!("文件已存在 > {}", out.display()));
                continue;
            }
            let src = video.location();
            fs::rename(src, &out).await?;
            info!("{}({}) > {}", src.display(), name, out.display());
            let msg = if *idx == 0 {
                format!("视频...{}", "ok".green())
            } else {
                format!("视频({idx})...{}", "ok".green())
            };
            self.bar.message(msg);
        }

        Ok(())
    }

    pub fn get_by_tag(&self, tag: &Tag) -> String {
        match tag {
            Tag::Title => self.nfo.title().to_string(),
            Tag::Studio => self.nfo.studio().to_string(),
            Tag::Id => self.video.ty().id().to_string(),
            Tag::Name => self.video.ty().name(),
            Tag::Director => self.nfo.director().to_string(),
            Tag::Country => self.nfo.country().to_string(),
            Tag::Actor => self
                .nfo
                .actors()
                .iter()
                .next()
                .map(|actor| actor.as_str())
                .unwrap_or("未知")
                .to_string(),
        }
    }

    pub async fn write_all_to(&self, path: &Path) -> Result<()> {
        self.write_fanart_to(path).await?;
        self.write_poster_to(path).await?;
        self.write_subtitle_to(path).await?;
        self.write_nfo_to(path).await?;

        Ok(())
    }
}
