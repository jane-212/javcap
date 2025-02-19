use std::collections::HashSet;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;

use bon::bon;
use educe::Educe;
use getset::{Getters, MutGetters, Setters};
use indoc::writedoc;
use quick_xml::escape::escape;
use validator::Validate;
use video::VideoType;

#[derive(Setters, Getters, MutGetters, Validate, Educe)]
#[educe(PartialEq)]
pub struct Nfo {
    id: String,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    title: String,

    #[getset(set = "pub")]
    rating: f64,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    plot: String,

    #[getset(set = "pub")]
    runtime: u32,

    mpaa: Mpaa,

    #[getset(get_mut = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    genres: HashSet<String>,

    #[getset(get = "pub")]
    country: Country,

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
    #[educe(PartialEq(ignore))]
    poster: Vec<u8>,

    #[getset(set = "pub", get = "pub")]
    #[validate(length(min = 1, message = "empty"))]
    #[educe(PartialEq(ignore))]
    fanart: Vec<u8>,

    #[getset(set = "pub", get = "pub")]
    #[educe(PartialEq(ignore))]
    subtitle: Vec<u8>,
}

#[bon]
impl Nfo {
    #[builder]
    pub fn new(id: impl Into<String>, country: Option<Country>, mpaa: Option<Mpaa>) -> Nfo {
        Nfo {
            id: id.into(),
            country: country.unwrap_or(Country::Unknown),
            mpaa: mpaa.unwrap_or(Mpaa::G),
            title: String::new(),
            rating: 0.0,
            plot: String::new(),
            runtime: 0,
            genres: HashSet::new(),
            director: String::new(),
            premiered: String::new(),
            studio: String::new(),
            actors: HashSet::new(),
            poster: Vec::new(),
            fanart: Vec::new(),
            subtitle: Vec::new(),
        }
    }

    pub fn auto_fix_by_key(&mut self, key: &VideoType) {
        if self.plot.is_empty() {
            self.plot = self.title.clone();
        }
        match key {
            VideoType::Jav(_, _) => {
                if self.director.is_empty() {
                    self.director = self.studio.clone();
                }
            }
            VideoType::Fc2(_) => {
                if self.studio.is_empty() {
                    self.studio = "FC2-PPV".to_string();
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
            VideoType::Other(_) => {
                if self.poster.is_empty() {
                    self.poster = self.fanart.clone();
                }
                if self.genres.is_empty() {
                    self.genres.insert(self.director.clone());
                }
                if self.actors.is_empty() {
                    self.actors.insert(self.director.clone());
                }
            }
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

impl Debug for Nfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            id: {}
            country: {}
            mpaa: {}
            title: {}
            rating: {}
            plot: {}
            runtime: {}
            genres: {}
            director: {}
            premiered: {}
            studio: {}
            actors: {}
            fanart: {}
            poster: {}
            subtitle: {}",
            self.id,
            self.country,
            self.mpaa,
            self.title,
            self.rating,
            self.plot,
            self.runtime,
            self.genres
                .iter()
                .map(|genre| genre.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            self.director,
            self.premiered,
            self.studio,
            self.actors
                .iter()
                .map(|actor| actor.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            self.fanart.len(),
            self.poster.len(),
            self.subtitle.len(),
        )
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
            title = escape(&self.title),
            rating = self.rating,
            plot = escape(&self.plot),
            runtime = self.runtime,
            mpaa = self.mpaa,
            id = self.id,
            genres = self
                .genres
                .iter()
                .map(|genre| format!("    <genre>{}</genre>", escape(genre)))
                .collect::<Vec<_>>()
                .join("\n"),
            tags = self
                .genres
                .iter()
                .map(|genre| format!("    <tag>{}</tag>", escape(genre)))
                .collect::<Vec<_>>()
                .join("\n"),
            country = self.country,
            director = escape(&self.director),
            premiered = self.premiered,
            studio = escape(&self.studio),
            actors = self
                .actors
                .iter()
                .map(|actor| format!(
                    "    <actor>\n        <name>{}</name>\n    </actor>",
                    escape(actor)
                ))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

#[derive(PartialEq, Eq)]
pub enum Country {
    Unknown,
    Japan,
    China,
}

impl Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Country::Unknown => "未知",
                Country::Japan => "日本",
                Country::China => "国产",
            }
        )
    }
}

impl Merge for Country {
    fn merge(&mut self, other: Self) {
        if Country::Unknown == *self {
            *self = other;
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Mpaa {
    G,
    PG,
    PG13,
    R,
    NC17,
}

impl Display for Mpaa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mpaa::G => "G",
                Mpaa::PG => "PG",
                Mpaa::PG13 => "PG-13",
                Mpaa::R => "R",
                Mpaa::NC17 => "NC-17",
            }
        )
    }
}

impl Merge for Mpaa {
    fn merge(&mut self, other: Self) {
        if *self < other {
            *self = other;
        }
    }
}
