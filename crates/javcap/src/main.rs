use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use config::Config;
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
    println!("当前版本: {}", app::VERSION);

    let config = Config::load().await?;
    config.validate()?;

    let videos = load_all_videos(&config).await?;
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

    for file in walk_dir(&config.input.path, &config.input.excludes)? {
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
            video.add_file(VideoFile::new(&file, idx));
        }
    }

    let videos = map.into_values().collect();
    Ok(videos)
}

fn walk_dir(path: &Path, excludes: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
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
            let child_files = walk_dir(&file, excludes)?;
            files.extend(child_files);
            continue;
        }

        files.push(file);
    }

    Ok(files)
}
