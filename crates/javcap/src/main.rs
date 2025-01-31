use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use config::Config;
use tokio::fs;
use validator::Validate;
use video::{Video, VideoFile, VideoType};

#[tokio::main]
async fn main() {
    println!("{}", ">".repeat(app::LINE_LENGTH));
    if let Err(e) = run().await {
        println!("{:#^width$}", " Error ", width = app::LINE_LENGTH);
        eprintln!("{}", e);
    }
    println!("{}", "<".repeat(app::LINE_LENGTH));
}

async fn run() -> Result<()> {
    println!("当前版本: {}({})", app::VERSION, app::HASH);

    let config = Config::load().await?;
    config.validate()?;

    let mut videos = load_all_videos(&config).await?;
    videos.sort_by_key(|v| v.ty().name());
    println!(
        "共找到视频: {}({})",
        videos.len(),
        videos
            .iter()
            .map(|video| video.ty().name())
            .collect::<Vec<_>>()
            .join(", ")
    );

    for video in videos {
        println!(
            "{:=^width$}",
            format!(" {} ", video.ty().name()),
            width = app::LINE_LENGTH
        );

        println!("{:#?}", video);
    }

    Ok(())
}

async fn load_all_videos(config: &Config) -> Result<Vec<Video>> {
    let mut map = HashMap::new();

    for file in walk_dir(&config.input.path, &config.input.excludes).await? {
        let name = match file.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };

        let (file_name, ext) = match name.split_once('.') {
            Some(res) => res,
            None => continue,
        };

        if config.input.exts.iter().any(|e| e == ext) {
            let (video_ty, idx) = match VideoType::parse(file_name) {
                Ok(res) => res,
                Err(_) => continue,
            };

            let video = map.entry(video_ty.clone()).or_insert(Video::new(video_ty));
            video.add_file(VideoFile::new(&file, ext, idx));
        }
    }

    let videos = map.into_values().collect();
    Ok(videos)
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
            let child_files = Box::pin(walk_dir(&file, excludes)).await?;
            files.extend(child_files);
            continue;
        }

        files.push(file);
    }

    Ok(files)
}
