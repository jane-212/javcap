use std::path::Path;

use anyhow::Result;
use config::Tag;
use getset::Getters;
use nfo::Nfo;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use video::Video;

#[derive(Getters)]
pub struct Payload {
    #[getset(get = "pub")]
    video: Video,
    nfo: Nfo,
}

impl Payload {
    pub fn new(video: Video, nfo: Nfo) -> Payload {
        Payload { video, nfo }
    }

    async fn write_fanart_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty().name();
        let filename = format!("{name}-fanart.jpg");
        let file = path.join(filename);
        Self::write_to_file(self.nfo.fanart(), &file).await?;
        println!("fanart已写入");

        Ok(())
    }

    async fn write_poster_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty().name();
        let filename = format!("{name}-poster.jpg");
        let file = path.join(filename);
        Self::write_to_file(self.nfo.poster(), &file).await?;
        println!("poster已写入");

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
        println!("nfo已写入");

        Ok(())
    }

    async fn write_subtitle_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty().name();
        let filename = format!("{name}.srt");
        let file = path.join(filename);
        Self::write_to_file(self.nfo.subtitle(), &file).await?;
        println!("字幕已写入");

        Ok(())
    }

    pub async fn move_videos_to(&self, path: &Path) -> Result<()> {
        for video in self.video.files() {
            let idx = video.idx();
            let filename = if *idx == 0 {
                format!("{}.{}", self.video.ty().name(), video.ext())
            } else {
                format!("{}-{}.{}", self.video.ty().name(), idx, video.ext())
            };
            let out = path.join(&filename);
            if out.exists() {
                println!("文件已存在 > {}", out.display());
                continue;
            }
            let src = video.location();
            fs::rename(src, &out).await?;
            println!("移动 {} > {}", src.display(), out.display());
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
