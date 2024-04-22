use std::path::{Path, PathBuf};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::{eof, map, opt},
    multi::many0,
    sequence::tuple,
    IResult,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VideoParser {
    FC2(String, PathBuf, u32),
    Normal(String, PathBuf, u32),
}

impl VideoParser {
    pub fn id(&self) -> &str {
        match self {
            Self::FC2(id, _, _) => id,
            Self::Normal(id, _, _) => id,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::FC2(_, path, _) => path,
            Self::Normal(_, path, _) => path,
        }
    }

    pub fn idx(&self) -> u32 {
        match self {
            Self::FC2(_, _, idx) => *idx,
            Self::Normal(_, _, idx) => *idx,
        }
    }

    pub fn matches(&self, id: &str) -> bool {
        let (id, num, _) = Self::parse_name(id)
            .map(|(_, id)| id)
            .unwrap_or(("", "", 0));

        self.id() == format!("{}-{}", id, num)
    }

    pub fn parse(path: &Path) -> anyhow::Result<Self> {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.to_uppercase())
            .unwrap_or("".to_string());
        let (_, (id, num, idx)) =
            Self::parse_name(&name).map_err(|_| anyhow::anyhow!("id not found in {name}"))?;
        let video = match id {
            "FC2-PPV" => Self::FC2(format!("{}-{}", id, num), path.to_path_buf(), idx),
            _ => Self::Normal(format!("{}-{}", id, num), path.to_path_buf(), idx),
        };

        Ok(video)
    }

    fn parse_name(input: &str) -> IResult<&str, (&str, &str, u32)> {
        map(
            tuple((
                take_while(|c: char| !c.is_ascii_alphabetic()),
                Self::name,
                take_while(|c: char| !c.is_ascii_digit()),
                eof,
            )),
            |(_, id, _, _)| id,
        )(input)
    }

    fn split(input: &str) -> IResult<&str, Vec<&str>> {
        many0(alt((tag("-"), tag(" "))))(input)
    }

    fn name(input: &str) -> IResult<&str, (&str, &str, u32)> {
        alt((Self::fc2, Self::normal))(input)
    }

    fn fc2(input: &str) -> IResult<&str, (&str, &str, u32)> {
        map(
            tuple((
                tag("FC2"),
                Self::split,
                opt(tag("PPV")),
                Self::split,
                take_while1(|c: char| c.is_ascii_digit()),
                Self::split,
                opt(tag("CD")),
                take_while(|c: char| c.is_ascii_digit()),
                take_while(|_| true),
            )),
            |(_, _, _, _, num, _, _, idx, _)| ("FC2-PPV", num, idx.parse::<u32>().unwrap_or(0)),
        )(input)
    }

    fn normal(input: &str) -> IResult<&str, (&str, &str, u32)> {
        map(
            tuple((
                take_while1(|c: char| c.is_ascii_alphabetic()),
                Self::split,
                take_while1(|c: char| c.is_ascii_digit()),
                Self::split,
                opt(tag("CD")),
                take_while(|c: char| c.is_ascii_digit()),
                take_while(|_| true),
            )),
            |(id, _, num, _, _, idx, _)| (id, num, idx.parse::<u32>().unwrap_or(0)),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_normal() {
        let pairs = vec![
            (
                "stars-804.mp4",
                Ok(VideoParser::Normal(
                    "STARS-804".to_string(),
                    PathBuf::from("stars-804.mp4"),
                    0,
                )),
            ),
            (
                "stars - 804.mp4",
                Ok(VideoParser::Normal(
                    "STARS-804".to_string(),
                    PathBuf::from("stars - 804.mp4"),
                    0,
                )),
            ),
            (
                "stars804.mp4",
                Ok(VideoParser::Normal(
                    "STARS-804".to_string(),
                    PathBuf::from("stars804.mp4"),
                    0,
                )),
            ),
            ("stars.mp4", Err("id not found in STARS".to_string())),
        ];

        for pair in pairs {
            let path = PathBuf::from(pair.0);
            let video = VideoParser::parse(&path).map_err(|err| err.to_string());
            assert_eq!(video, pair.1)
        }
    }

    #[test]
    fn test_fc2() {
        let pairs = vec![
            (
                "fc2-ppv-3234.mp4",
                Ok(VideoParser::FC2(
                    "FC2-PPV-3234".to_string(),
                    PathBuf::from("fc2-ppv-3234.mp4"),
                    0,
                )),
            ),
            (
                "fc2ppv3234.mp4",
                Ok(VideoParser::FC2(
                    "FC2-PPV-3234".to_string(),
                    PathBuf::from("fc2ppv3234.mp4"),
                    0,
                )),
            ),
            (
                "fc2-3234.mp4",
                Ok(VideoParser::FC2(
                    "FC2-PPV-3234".to_string(),
                    PathBuf::from("fc2-3234.mp4"),
                    0,
                )),
            ),
            (
                "fc23234.mp4",
                Ok(VideoParser::FC2(
                    "FC2-PPV-3234".to_string(),
                    PathBuf::from("fc23234.mp4"),
                    0,
                )),
            ),
            (
                "fc2 - 3234.mp4",
                Ok(VideoParser::FC2(
                    "FC2-PPV-3234".to_string(),
                    PathBuf::from("fc2 - 3234.mp4"),
                    0,
                )),
            ),
            (
                "fc2 - ppv - 3234.mp4",
                Ok(VideoParser::FC2(
                    "FC2-PPV-3234".to_string(),
                    PathBuf::from("fc2 - ppv - 3234.mp4"),
                    0,
                )),
            ),
            (
                "fc2-ppv-3234-1.mp4",
                Ok(VideoParser::FC2(
                    "FC2-PPV-3234".to_string(),
                    PathBuf::from("fc2-ppv-3234-1.mp4"),
                    1,
                )),
            ),
            (
                "fc2-ppv-3234-2.mp4",
                Ok(VideoParser::FC2(
                    "FC2-PPV-3234".to_string(),
                    PathBuf::from("fc2-ppv-3234-2.mp4"),
                    2,
                )),
            ),
            (
                "fc2-3234-2.mp4",
                Ok(VideoParser::FC2(
                    "FC2-PPV-3234".to_string(),
                    PathBuf::from("fc2-3234-2.mp4"),
                    2,
                )),
            ),
        ];

        for pair in pairs {
            let path = PathBuf::from(pair.0);
            let video = VideoParser::parse(&path).map_err(|err| err.to_string());
            assert_eq!(video, pair.1)
        }
    }
}
