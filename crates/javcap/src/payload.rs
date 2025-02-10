use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
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
        let name = self.video.ty();
        let filename = format!("{name}-fanart.jpg");
        let file = path.join(filename);
        Self::write_to_file(self.nfo.fanart(), &file)
            .await
            .with_context(|| format!("write to file {}", file.display()))?;
        info!("write fanart of {name} to {}", file.display());
        self.bar.message(format!("fanart...{}", "ok".green()));

        Ok(())
    }

    async fn write_poster_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty();
        let filename = format!("{name}-poster.jpg");
        let file = path.join(filename);
        Self::write_to_file(self.nfo.poster(), &file)
            .await
            .with_context(|| format!("write to file {}", file.display()))?;
        info!("write poster of {name} to {}", file.display());
        self.bar.message(format!("poster...{}", "ok".green()));

        Ok(())
    }

    async fn write_to_file(bytes: &[u8], file: &Path) -> Result<()> {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file)
            .await
            .with_context(|| format!("open {}", file.display()))?
            .write_all(bytes)
            .await
            .with_context(|| "write content")?;

        Ok(())
    }

    async fn write_nfo_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty();
        let filename = format!("{name}.nfo");
        let file = path.join(filename);
        let nfo = self.nfo.to_string();
        Self::write_to_file(nfo.as_bytes(), &file)
            .await
            .with_context(|| format!("write to file {}", file.display()))?;
        info!("write nfo of {name} to {}", file.display());
        self.bar.message(format!("nfo...{}", "ok".green()));

        Ok(())
    }

    async fn write_subtitle_to(&self, path: &Path) -> Result<()> {
        if self.nfo.subtitle().is_empty() {
            return Ok(());
        }

        let name = self.video.ty();
        let filename = format!("{name}.srt");
        let file = path.join(filename);
        Self::write_to_file(self.nfo.subtitle(), &file)
            .await
            .with_context(|| format!("write to file {}", file.display()))?;
        info!("write subtitle of {name} to {}", file.display());
        self.bar.message(format!("subtitle...{}", "ok".green()));

        Ok(())
    }

    pub async fn move_videos_to(&self, path: &Path) -> Result<()> {
        let name = self.video.ty();
        for video in self.video.files() {
            let idx = video.idx();
            let filename = if *idx == 0 {
                format!("{name}.{}", video.ext())
            } else {
                format!("{name}-CD{idx}.{}", video.ext())
            };
            let out = path.join(&filename);
            if out.exists() {
                info!("video already exists {}", out.display());
                self.bar
                    .message(format!("video already exists {}", out.display()));
                continue;
            }
            let src = video.location();
            fs::rename(src, &out).await?;
            info!(
                "move video of {name} from {} to {}",
                src.display(),
                out.display()
            );
            let msg = if *idx == 0 {
                format!("video...{}", "ok".green())
            } else {
                format!("video({idx})...{}", "ok".green())
            };
            self.bar.message(msg);
        }

        Ok(())
    }

    pub fn get_by_tag(&self, tag: &Tag) -> String {
        match tag {
            Tag::Title => self.nfo.title().to_string(),
            Tag::Studio => self.nfo.studio().to_string(),
            Tag::Id => match self.video.ty() {
                video::VideoType::Jav(id, _) => id.to_string(),
                video::VideoType::Fc2(_) => "FC2-PPV".to_string(),
            },
            Tag::Name => self.video.ty().to_string(),
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
        self.write_fanart_to(path)
            .await
            .with_context(|| "write fanart")?;
        self.write_poster_to(path)
            .await
            .with_context(|| "write poster")?;
        self.write_subtitle_to(path)
            .await
            .with_context(|| "write subtitle")?;
        self.write_nfo_to(path).await.with_context(|| "write nfo")?;

        Ok(())
    }
}
