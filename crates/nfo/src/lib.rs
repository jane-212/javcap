use std::collections::HashSet;
use std::fmt::{self, Display};
use std::hash::Hash;

use getset::{Getters, MutGetters, Setters};
use indoc::{formatdoc, writedoc};
use validator::Validate;

#[derive(Default, Setters, Getters, MutGetters, Validate)]
pub struct Nfo {
    #[getset(get = "pub")]
    id: String,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    title: String,

    #[getset(set = "pub")]
    #[validate(range(min = 0.1, message = "empty"))]
    rating: f64,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    plot: String,

    #[getset(set = "pub")]
    #[validate(range(min = 1, message = "empty"))]
    runtime: u32,

    #[getset(set = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    mpaa: String,

    #[getset(get_mut = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    genres: HashSet<String>,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    country: String,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    director: String,

    #[getset(set = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    premiered: String,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    studio: String,

    #[getset(get_mut = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    actors: HashSet<String>,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    poster: Vec<u8>,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    fanart: Vec<u8>,

    #[getset(set = "pub", get = "pub")]
    subtitle: Vec<u8>,
}

impl Nfo {
    pub fn new(id: impl Into<String>) -> Nfo {
        Nfo {
            id: id.into(),
            ..Default::default()
        }
    }

    pub fn auto_fix(&mut self) {
        self.rating = self.rating.max(0.1);
        if self.plot.is_empty() {
            self.plot = self.title.clone();
        }
        if self.director.is_empty() {
            self.director = self.studio.clone();
        }
        if self.genres.is_empty() {
            let director = self.director.clone();
            self.genres_mut().insert(director);
        }
        if self.actors.is_empty() {
            let director = self.director.clone();
            self.actors_mut().insert(director);
        }
    }

    pub fn summary(&self) -> String {
        formatdoc!(
            "
            {self}
            fanart: {}
            poster: {}
            subtitle: {}",
            self.fanart.len(),
            self.poster.len(),
            self.subtitle.len(),
        )
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

impl Merge for Vec<u8> {
    fn merge(&mut self, other: Self) {
        if self.len() < other.len() {
            *self = other;
        }
    }
}

impl Merge for u32 {
    fn merge(&mut self, other: Self) {
        if *self < other {
            *self = other;
        }
    }
}

impl Merge for f64 {
    fn merge(&mut self, other: Self) {
        if *self < other {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            <?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?>
            <movie>
                <title>{title}</title>
                <originaltitle>{title}</originaltitle>
                <rating>{rating:.1}</rating>
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
            </movie>",
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
