use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

use getset::{Getters, Setters};
use indoc::writedoc;
use validator::Validate;

#[derive(Default, Setters, Getters, Validate)]
pub struct Nfo {
    id: String,

    #[getset(set = "pub")]
    #[validate(length(min = 1, message = "空"))]
    title: String,

    #[getset(set = "pub")]
    #[validate(range(min = 0.1, message = "无评分"))]
    rating: f64,

    #[getset(set = "pub")]
    #[validate(length(min = 1, message = "空"))]
    plot: String,

    #[getset(set = "pub")]
    #[validate(range(min = 1, message = "无时长"))]
    runtime: u32,

    #[getset(set = "pub")]
    #[validate(length(min = 1, message = "空"))]
    mpaa: String,

    #[getset(get_mut = "pub")]
    genres: HashSet<String>,

    #[getset(set = "pub")]
    #[validate(length(min = 1, message = "空"))]
    country: String,

    #[getset(set = "pub")]
    #[validate(length(min = 1, message = "空"))]
    director: String,

    #[getset(set = "pub")]
    #[validate(length(min = 1, message = "空"))]
    premiered: String,

    #[getset(set = "pub")]
    #[validate(length(min = 1, message = "空"))]
    studio: String,

    #[getset(get_mut = "pub")]
    actors: HashSet<String>,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "空"))]
    poster: Vec<u8>,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "空"))]
    fanart: Vec<u8>,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "空"))]
    subtitle: Vec<u8>,
}

impl Nfo {
    pub fn new(id: impl Into<String>) -> Nfo {
        Nfo {
            id: id.into(),
            ..Default::default()
        }
    }

    pub fn merge(&mut self, other: Nfo) {
        self.title.merge(other.title);
        self.rating.merge(other.rating);
        self.plot.merge(other.plot);
        self.runtime.merge(other.runtime);
        self.mpaa.merge(other.mpaa);
        self.genres.merge(other.genres);
        self.country.merge(other.country);
        self.director.merge(other.director);
        self.premiered.merge(other.premiered);
        self.studio.merge(other.studio);
        self.actors.merge(other.actors);
        self.poster.merge(other.poster);
        self.fanart.merge(other.fanart);
        self.subtitle.merge(other.subtitle);
    }
}

trait Merge {
    fn merge(&mut self, other: Self);
}

impl Merge for f64 {
    fn merge(&mut self, other: Self) {
        if *self == 0.0 {
            *self = other;
        }
    }
}

impl Merge for Vec<u8> {
    fn merge(&mut self, other: Self) {
        if self.len() < other.len() {
            *self = other;
        }
    }
}

impl Merge for u32 {
    fn merge(&mut self, other: Self) {
        if *self == 0 {
            *self = other;
        }
    }
}

impl<T: Hash + Eq> Merge for HashSet<T> {
    fn merge(&mut self, other: Self) {
        self.extend(other);
    }
}

impl Merge for String {
    fn merge(&mut self, other: Self) {
        if self.len() < other.len() {
            *self = other;
        }
    }
}

impl Display for Nfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writedoc!(
            f,
            "
            <?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?>
            <movie>
                <title>{title}</title>
                <originaltitle>{title}</originaltitle>
                <rating>{rating}</rating>
                <plot>{plot}</plot>
                <runtime>{runtime}</runtime>
                <mpaa>{mpaa}</mpaa>
                <uniqueid type=\"num\" default=\"true\">{id}</uniqueid>
            {genres}
            {tags}
                <country>{country}</country>
                <director>{director}</director>
                <premiered>{premiered}</premiered>
                <studio>{studio}</studio>
            {actors}
            </movie>
            ",
            title = self.title,
            rating = self.rating,
            plot = self.plot,
            runtime = self.runtime,
            mpaa = self.mpaa,
            id = self.id,
            genres = self
                .genres
                .iter()
                .map(|genre| format!("    <genre>{genre}</genre>"))
                .collect::<Vec<String>>()
                .join("\n"),
            tags = self
                .genres
                .iter()
                .map(|genre| format!("    <tag>{genre}</tag>"))
                .collect::<Vec<String>>()
                .join("\n"),
            country = self.country,
            director = self.director,
            premiered = self.premiered,
            studio = self.studio,
            actors = self
                .actors
                .iter()
                .map(|actor| format!("    <actor>\n        <name>{actor}</name>\n    </actor>"))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}
