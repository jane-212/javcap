use indoc::formatdoc;
use serde::Serialize;
use std::{collections::HashSet, hash::Hash, path::Path};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};
use tracing::info;

use crate::Video;

#[derive(Default, Serialize)]
pub struct Info {
    title: String,
    rating: f64,
    plot: String,
    runtime: u32,
    mpaa: String,
    id: String,
    genres: Vec<String>,
    country: String,
    director: String,
    premiered: String,
    studio: String,
    actors: Vec<String>,
    poster: Vec<u8>,
    fanart: Vec<u8>,
}

const MOVIE_NFO: &str = "movie.nfo";

impl Info {
    pub fn new(id: String) -> Info {
        Info {
            mpaa: "NC-17".to_string(),
            country: "日本".to_string(),
            id,
            ..Default::default()
        }
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_plot(&self) -> &str {
        &self.plot
    }

    fn to_nfo(&self) -> String {
        formatdoc!(
            "
            <?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?>
            <movie>
                <title>{title}</title>
                <originaltitle>{title}</originaltitle>
                <rating>{rating}</rating>
                <plot>{plot}</plot>
                <runtime>{runtime}</runtime>
                <mpaa>{mpaa}</mpaa>
                <uniqueid type=\"num\" defult=\"true\">{id}</uniqueid>
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

    pub async fn write_to(self, path: &Path, file: &Path) -> anyhow::Result<()> {
        let path = path.join(&self.studio).join(&self.id);
        let ext = file.extension().and_then(|ext| ext.to_str());
        let to_file = match ext {
            Some(ext) => format!("{}.{}", self.id, ext),
            None => self.id.to_string(),
        };
        if path.join(&to_file).exists() {
            anyhow::bail!("video {} already exists", self.id);
        }
        fs::create_dir_all(&path).await?;
        info!("create {}", path.display());
        if !self.poster.is_empty() {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path.join("poster.jpg"))
                .await?
                .write_all(&self.poster)
                .await?;
            info!("write poster.jpg to {}", path.join("poster.jpg").display());
        }
        if !self.fanart.is_empty() {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path.join("fanart.jpg"))
                .await?
                .write_all(&self.fanart)
                .await?;
            info!("write fanart.jpg to {}", path.join("fanart.jpg").display());
        }
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.join(MOVIE_NFO))
            .await?
            .write_all(self.to_nfo().as_bytes())
            .await?;
        info!("write {} to {}", MOVIE_NFO, path.join(MOVIE_NFO).display());
        fs::rename(file, path.join(&to_file)).await?;
        info!(
            "move {} to {}",
            file.display(),
            path.join(&to_file).display()
        );

        Ok(())
    }

    fn fix_normal(&mut self) {
        if self.plot.is_empty() {
            self.plot = self.title.clone();
        }
        if self.director.is_empty() {
            self.director = self.studio.clone();
        }
    }

    fn fix_fc2(&mut self) {
        if self.plot.is_empty() {
            self.plot = self.title.clone();
        }
        if self.actors.is_empty() {
            self.actors.push(self.director.clone());
        }
    }

    fn check_normal(self) -> Option<Info> {
        if self.title.is_empty() {
            return None;
        }
        if self.plot.is_empty() {
            return None;
        }
        if self.runtime == 0 {
            return None;
        }
        if self.genres.is_empty() {
            return None;
        }
        if self.director.is_empty() {
            return None;
        }
        if self.premiered.is_empty() {
            return None;
        }
        if self.studio.is_empty() {
            return None;
        }
        if self.actors.is_empty() {
            return None;
        }
        if self.poster.is_empty() {
            return None;
        }
        if self.fanart.is_empty() {
            return None;
        }

        Some(self)
    }

    fn check_fc2(self) -> Option<Info> {
        if self.title.is_empty() {
            return None;
        }
        if self.plot.is_empty() {
            return None;
        }
        if self.runtime == 0 {
            return None;
        }
        if self.director.is_empty() {
            return None;
        }
        if self.premiered.is_empty() {
            return None;
        }
        if self.studio.is_empty() {
            return None;
        }
        if self.actors.is_empty() {
            return None;
        }
        if self.fanart.is_empty() {
            return None;
        }

        Some(self)
    }

    pub fn check(mut self, video: &Video) -> Option<Info> {
        match video {
            Video::FC2(_, _) => {
                self.fix_fc2();
                self.check_fc2()
            }
            Video::Normal(_, _) => {
                self.fix_normal();
                self.check_normal()
            }
        }
    }

    fn combine_vec<T: Eq + Hash + Clone>(left: &[T], right: &[T]) -> Vec<T> {
        let mut hset = HashSet::new();
        for item in left {
            hset.insert(item.clone());
        }
        for item in right {
            hset.insert(item.clone());
        }

        hset.into_iter().collect()
    }

    fn select_long(left: &str, right: &str) -> String {
        if left.len() > right.len() {
            left.to_string()
        } else {
            right.to_string()
        }
    }

    #[cfg(debug_assertions)]
    fn empty_print(s: &str) -> &str {
        if s.is_empty() {
            "<empty>"
        } else {
            s
        }
    }

    #[cfg(debug_assertions)]
    pub fn show_info(&self, id: &str) {
        let empty_print = Info::empty_print;
        info!("{:-^25}", format!(" {} BEGIN ", id));
        info!("title: {}", empty_print(&self.title));
        info!("rating: {}", self.rating);
        info!("plot: {}", empty_print(&self.plot));
        info!("runtime: {}", self.runtime);
        info!("genres: {:#?}", self.genres);
        info!("director: {}", empty_print(&self.director));
        info!("premiered: {}", empty_print(&self.premiered));
        info!("studio: {}", empty_print(&self.studio));
        info!("actors: {:#?}", self.actors);
        info!(
            "poster: {}",
            if self.poster.is_empty() { "no" } else { "yes" }
        );
        info!(
            "fanart: {}",
            if self.fanart.is_empty() { "no" } else { "yes" }
        );
        info!("{:-^25}", format!(" {} END ", id));
    }

    pub fn merge(&mut self, other: Info) {
        if self.title.is_empty() {
            self.title = Info::select_long(&self.title, &other.title);
        }
        if self.rating == 0.0 {
            self.rating = other.rating;
        }
        if self.plot.is_empty() {
            self.plot = Info::select_long(&self.plot, &other.plot);
        }
        if self.runtime == 0 {
            self.runtime = other.runtime;
        }
        self.genres = Info::combine_vec(&self.genres, &other.genres);
        if self.director.is_empty() {
            self.director = Info::select_long(&self.director, &other.director);
        }
        if self.premiered.is_empty() {
            self.premiered = Info::select_long(&self.premiered, &other.premiered);
        }
        if self.studio.is_empty() {
            self.studio = Info::select_long(&self.studio, &other.studio);
        }
        self.actors = Info::combine_vec(&self.actors, &other.actors);
        if self.poster.len() < other.poster.len() {
            self.poster = other.poster;
        }
        if self.fanart.len() < other.fanart.len() {
            self.fanart = other.fanart;
        }
    }

    pub fn title(&mut self, title: String) {
        self.title = title;
    }

    pub fn poster(&mut self, poster: Vec<u8>) {
        self.poster = poster;
    }

    pub fn fanart(&mut self, fanart: Vec<u8>) {
        self.fanart = fanart;
    }

    pub fn rating(&mut self, rating: f64) {
        self.rating = rating;
    }

    pub fn plot(&mut self, plot: String) {
        self.plot = plot;
    }

    pub fn runtime(&mut self, runtime: u32) {
        self.runtime = runtime;
    }

    pub fn genres(&mut self, genres: Vec<String>) {
        self.genres = genres;
    }

    pub fn director(&mut self, director: String) {
        self.director = director;
    }

    pub fn premiered(&mut self, premiered: String) {
        self.premiered = premiered;
    }

    pub fn studio(&mut self, studio: String) {
        self.studio = studio;
    }

    pub fn actors(&mut self, actors: Vec<String>) {
        self.actors = actors;
    }
}
