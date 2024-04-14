use std::path::{Path, PathBuf};

use error::{Error, Result};
use nom::{
    branch::alt,
    bytes::{
        complete::take_while,
        streaming::{tag, take_while1},
    },
    combinator::{eof, map, opt},
    multi::many0,
    sequence::tuple,
    IResult,
};

#[derive(Clone)]
pub enum Video {
    FC2(String, PathBuf),
    Normal(String, PathBuf),
}

impl Video {
    pub fn id(&self) -> &str {
        match self {
            Video::FC2(id, _) => id,
            Video::Normal(id, _) => id,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Video::FC2(_, path) => path,
            Video::Normal(_, path) => path,
        }
    }

    pub fn matches(&self, id: &str) -> bool {
        let (id, num) = Video::parse_name(id).map(|(_, id)| id).unwrap_or(("", ""));

        self.id() == format!("{}-{}", id, num)
    }

    pub fn parse(path: &Path) -> Result<Video> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.rsplit_once('.'))
            .map(|name| name.0.to_uppercase())
            .unwrap_or("".to_string());
        let (_, (id, num)) = Video::parse_name(&name).map_err(|_| {
            Error::Parse(
                path.file_name()
                    .and_then(|name| name.to_str().map(|name| name.to_string()))
                    .unwrap_or("-".to_string()),
            )
        })?;
        let video = match id {
            "FC2-PPV" => Video::FC2(format!("{}-{}", id, num), path.to_path_buf()),
            _ => Video::Normal(format!("{}-{}", id, num), path.to_path_buf()),
        };

        Ok(video)
    }

    fn parse_name(input: &str) -> IResult<&str, (&str, &str)> {
        map(
            tuple((
                take_while(|c: char| !c.is_ascii_alphanumeric()),
                Video::name,
                take_while(|c: char| !c.is_ascii_alphanumeric()),
                eof,
            )),
            |(_, id, _, _)| id,
        )(input)
    }

    fn split(input: &str) -> IResult<&str, Vec<&str>> {
        many0(alt((tag("-"), tag(" "))))(input)
    }

    fn name(input: &str) -> IResult<&str, (&str, &str)> {
        alt((Video::fc2, Video::normal))(input)
    }

    fn fc2(input: &str) -> IResult<&str, (&str, &str)> {
        map(
            tuple((
                tag("FC2"),
                Video::split,
                opt(tag("PPV")),
                Video::split,
                take_while(|c: char| c.is_ascii_digit()),
            )),
            |(_, _, _, _, num)| ("FC2-PPV", num),
        )(input)
    }

    fn normal(input: &str) -> IResult<&str, (&str, &str)> {
        map(
            tuple((
                take_while1(|c: char| c.is_ascii_alphabetic()),
                Video::split,
                take_while(|c: char| c.is_ascii_digit()),
            )),
            |(id, _, num)| (id, num),
        )(input)
    }
}
