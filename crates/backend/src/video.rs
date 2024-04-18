use std::path::{Path, PathBuf};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::{eof, map, opt},
    multi::many0,
    sequence::tuple,
    IResult,
};

#[derive(Clone)]
pub enum Video {
    FC2(String, PathBuf, u32),
    Normal(String, PathBuf, u32),
}

impl Video {
    pub fn id(&self) -> &str {
        match self {
            Video::FC2(id, _, _) => id,
            Video::Normal(id, _, _) => id,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Video::FC2(_, path, _) => path,
            Video::Normal(_, path, _) => path,
        }
    }

    pub fn idx(&self) -> u32 {
        match self {
            Video::FC2(_, _, idx) => *idx,
            Video::Normal(_, _, idx) => *idx,
        }
    }

    pub fn matches(&self, id: &str) -> bool {
        let (id, num, _) = Video::parse_name(id)
            .map(|(_, id)| id)
            .unwrap_or(("", "", 0));

        self.id() == format!("{}-{}", id, num)
    }

    pub fn parse(path: &Path) -> anyhow::Result<Video> {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.to_uppercase())
            .unwrap_or("".to_string());
        let (_, (id, num, idx)) =
            Self::parse_name(&name).map_err(|_| anyhow::anyhow!("id not found in {name}"))?;
        let video = match id {
            "FC2-PPV" => Video::FC2(format!("{}-{}", id, num), path.to_path_buf(), idx),
            _ => Video::Normal(format!("{}-{}", id, num), path.to_path_buf(), idx),
        };

        Ok(video)
    }

    fn parse_name(input: &str) -> IResult<&str, (&str, &str, u32)> {
        map(
            tuple((
                take_while(|c: char| !c.is_ascii_alphabetic()),
                Video::name,
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
        alt((Video::fc2, Video::normal))(input)
    }

    fn fc2(input: &str) -> IResult<&str, (&str, &str, u32)> {
        map(
            tuple((
                tag("FC2"),
                Video::split,
                opt(tag("PPV")),
                Video::split,
                take_while1(|c: char| c.is_ascii_digit()),
                Video::split,
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
                Video::split,
                take_while1(|c: char| c.is_ascii_digit()),
                Video::split,
                opt(tag("CD")),
                take_while(|c: char| c.is_ascii_digit()),
                take_while(|_| true),
            )),
            |(id, _, num, _, _, idx, _)| (id, num, idx.parse::<u32>().unwrap_or(0)),
        )(input)
    }
}
