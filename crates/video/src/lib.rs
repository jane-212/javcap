use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

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
    Other(String),
}

impl Display for VideoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoType::Jav(id, number) => write!(f, "{id}-{number}"),
            VideoType::Fc2(number) => write!(f, "FC2-PPV-{number}"),
            VideoType::Other(title) => write!(f, "{title}"),
        }
    }
}

impl From<VideoType> for String {
    fn from(value: VideoType) -> Self {
        value.to_string()
    }
}

impl From<&VideoType> for String {
    fn from(value: &VideoType) -> Self {
        value.to_string()
    }
}

impl VideoType {
    /// parse given name to a video and idx
    ///
    /// # Examples
    ///
    /// ```
    /// use video::VideoType;
    ///
    /// let expected = VideoType::Jav("XXX".to_string(), "123".to_string());
    /// let (video, idx) = VideoType::parse("xxx-123");
    /// assert_eq!(expected, video);
    /// assert_eq!(idx, 0);
    /// ```
    pub fn parse(name: impl AsRef<str>) -> (VideoType, u32) {
        let name = name.as_ref().to_uppercase();

        let (ty, idx) = match Self::_parse(&name) {
            Ok((_, (id, key, idx))) => match id {
                "FC2-PPV" => (Self::fc2(key), idx),
                _ => (Self::jav(id, key), idx),
            },
            Err(_) => (Self::other(name.clone()), 0),
        };
        info!("parse {name} to {ty}-{idx}");

        (ty, idx)
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

    fn other(key: impl Into<String>) -> VideoType {
        VideoType::Other(key.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    #[test_case("stars-804", VideoType::Jav("STARS".to_string(), "804".to_string()), 0; "stars-804")]
    #[test_case("stars804", VideoType::Jav("STARS".to_string(), "804".to_string()), 0; "stars804")]
    #[test_case("stars804-1", VideoType::Jav("STARS".to_string(), "804".to_string()), 1; "stars804-1")]
    #[test_case("stars804-2", VideoType::Jav("STARS".to_string(), "804".to_string()), 2; "stars804-2")]
    #[test_case("stars-804-1", VideoType::Jav("STARS".to_string(), "804".to_string()), 1; "stars-804-1")]
    #[test_case("ipx-443-1", VideoType::Jav("IPX".to_string(), "443".to_string()), 1; "ipx-443-1")]
    #[test_case("ipx-443-2", VideoType::Jav("IPX".to_string(), "443".to_string()), 2; "ipx-443-2")]
    #[test_case("ipx443-3", VideoType::Jav("IPX".to_string(), "443".to_string()), 3; "ipx443-3")]
    #[test_case("fc2-123456", VideoType::Fc2("123456".to_string()), 0; "fc2-123456")]
    #[test_case("fc2ppv-123456", VideoType::Fc2("123456".to_string()), 0; "fc2ppv-123456")]
    #[test_case("fc2-ppv-123456", VideoType::Fc2("123456".to_string()), 0; "fc2-ppv-123456")]
    #[test_case("fc2-ppv-12345-1", VideoType::Fc2("12345".to_string()), 1; "fc2-ppv-12345-1")]
    #[test_case("fc2ppv-12345-2", VideoType::Fc2("12345".to_string()), 2; "fc2ppv-12345-2")]
    #[test_case("fc2-12345-3", VideoType::Fc2("12345".to_string()), 3; "fc2-12345-3")]
    #[test_case("fc212345-4", VideoType::Fc2("12345".to_string()), 4; "fc212345-4")]
    #[test_case("小飞棍来喽", VideoType::Other("小飞棍来喽".to_string()), 0; "小飞棍来喽")]
    fn test_parse(name: &str, video: VideoType, idx: u32) {
        let (actual_video, actual_idx) = VideoType::parse(name);
        assert_eq!(actual_video, video);
        assert_eq!(actual_idx, idx);
    }
}
