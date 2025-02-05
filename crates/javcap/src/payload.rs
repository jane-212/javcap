use std::path::Path;

use anyhow::Result;
use getset::Getters;
use nfo::Nfo;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use video::Video;

#[derive(Getters)]
pub struct Payload {
    #[getset(get = "pub")]
    video: Video,
    #[getset(get = "pub")]
    nfo: Nfo,
}

impl Payload {
    pub fn new(video: Video, nfo: Nfo) -> Payload {
        Payload { video, nfo }
    }

    pub async fn write_fanart_to(&self, path: &Path) -> Result<()> {
        let name = self.video().ty().name();
        let filename = format!("{name}-fanart.jpg");
        let file = path.join(filename);
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file)
            .await?
            .write_all(self.nfo().fanart())
            .await?;
        println!("fanart已写入");

        Ok(())
    }

    pub async fn write_nfo_to(&self, path: &Path) -> Result<()> {
        let name = self.video().ty().name();
        let filename = format!("{name}.nfo");
        let file = path.join(filename);
        let nfo = self.nfo().to_string();
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(file)
            .await?
            .write_all(nfo.as_bytes())
            .await?;
        println!("nfo已写入");

        Ok(())
    }
}
