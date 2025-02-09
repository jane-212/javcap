use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bon::bon;
use getset::Getters;
use log::info;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::{eof, map, opt},
    multi::many0,
    IResult, Parser,
};

#[derive(Debug, Getters, Clone)]
pub struct Video {
    #[getset(get = "pub")]
    ty: VideoType,
    #[getset(get = "pub")]
    files: Vec<VideoFile>,
}

impl Video {
    pub fn new(ty: VideoType) -> Video {
        Video {
            ty,
            files: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: VideoFile) {
        self.files.push(file);
    }
}

#[derive(Debug, Getters, Clone)]
pub struct VideoFile {
    #[getset(get = "pub")]
    location: PathBuf,
    #[getset(get = "pub")]
    ext: String,
    #[getset(get = "pub")]
    idx: u32,
}

#[bon]
impl VideoFile {
    #[builder]
    pub fn new(location: &Path, ext: impl Into<String>, idx: u32) -> VideoFile {
        VideoFile {
            location: location.to_path_buf(),
            ext: ext.into(),
            idx,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum VideoType {
    Jav(String, String),
    Fc2(String),
}

impl VideoType {
    /// jav -> xxx-123
    ///
    /// fc2 -> FC2-PPV-123
    pub fn name(&self) -> String {
        match self {
            VideoType::Jav(id, key) => format!("{id}-{key}"),
            VideoType::Fc2(key) => format!("FC2-PPV-{key}"),
        }
    }

    /// jav -> xxx
    ///
    /// fc2 -> FC2-PPV
    pub fn id(&self) -> &str {
        match self {
            VideoType::Jav(id, _) => id,
            VideoType::Fc2(_) => "FC2-PPV",
        }
    }

    /// jav -> xxx-123 | xxx 123 | xxx123 | xxx-123-1
    ///
    /// fc2 -> fc2-123 | fc2ppv-123 | fc2-ppv-123 | fc2ppv123 | fc2-ppv-123-1
    pub fn parse(name: impl AsRef<str>) -> Result<(VideoType, u32)> {
        let name = name.as_ref().to_uppercase();

        let (_, (id, key, idx)) = Self::_parse(&name).map_err(|err| anyhow!("{err}"))?;
        info!("parse {name} to {id}-{key}-{idx}");

        let ty = match id {
            "FC2-PPV" => Self::fc2(key),
            _ => Self::jav(id, key),
        };

        Ok((ty, idx))
    }

    fn _parse(input: &str) -> IResult<&str, (&str, &str, u32)> {
        map(
            (
                take_while(|c: char| !c.is_ascii_alphabetic()),
                Self::parse_name,
                take_while(|c: char| !c.is_ascii_digit()),
                eof,
            ),
            |(_, id, _, _)| id,
        )
        .parse(input)
    }

    fn split(input: &str) -> IResult<&str, Vec<&str>> {
        many0(alt((tag("-"), tag(" ")))).parse(input)
    }

    fn parse_name(input: &str) -> IResult<&str, (&str, &str, u32)> {
        alt((Self::parse_fc2, Self::parse_jav)).parse(input)
    }

    fn parse_fc2(input: &str) -> IResult<&str, (&str, &str, u32)> {
        map(
            (
                tag("FC2"),
                Self::split,
                opt(tag("PPV")),
                Self::split,
                take_while1(|c: char| c.is_ascii_digit()),
                Self::split,
                opt(tag("CD")),
                take_while(|c: char| c.is_ascii_digit()),
                take_while(|_| true),
            ),
            |(_, _, _, _, num, _, _, idx, _)| ("FC2-PPV", num, idx.parse::<u32>().unwrap_or(0)),
        )
        .parse(input)
    }

    fn parse_jav(input: &str) -> IResult<&str, (&str, &str, u32)> {
        map(
            (
                take_while1(|c: char| c.is_ascii_alphabetic()),
                Self::split,
                take_while1(|c: char| c.is_ascii_digit()),
                Self::split,
                opt(tag("CD")),
                take_while(|c: char| c.is_ascii_digit()),
                take_while(|_| true),
            ),
            |(id, _, num, _, _, idx, _)| (id, num, idx.parse::<u32>().unwrap_or(0)),
        )
        .parse(input)
    }

    fn jav(id: impl Into<String>, key: impl Into<String>) -> VideoType {
        VideoType::Jav(id.into(), key.into())
    }

    fn fc2(key: impl Into<String>) -> VideoType {
        VideoType::Fc2(key.into())
    }
}
