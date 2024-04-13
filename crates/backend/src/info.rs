use error::{Error, Result};
use serde::Serialize;
use std::{collections::HashSet, hash::Hash, path::Path, sync::OnceLock};
use tera::{Context, Tera};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};
use tracing::info;

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

fn movie() -> &'static Tera {
    static TERA: OnceLock<Tera> = OnceLock::new();
    TERA.get_or_init(|| {
        let mut tera = Tera::default();
        tera.add_raw_template(
            MOVIE_NFO,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/movie.nfo")),
        )
        .expect("add template error");
        tera
    })
}

impl Info {
    pub fn new() -> Info {
        Info {
            mpaa: "NC-17".to_string(),
            country: "日本".to_string(),
            ..Default::default()
        }
    }

    fn to_nfo(&self) -> Result<String> {
        Ok(movie().render(MOVIE_NFO, &Context::from_serialize(self)?)?)
    }

    pub async fn write_to(self, path: &Path, file: &Path) -> Result<()> {
        let path = path.join(&self.studio).join(&self.id);
        let ext = file.extension().and_then(|ext| ext.to_str());
        let to_file = match ext {
            Some(ext) => format!("{}.{}", self.id, ext),
            None => self.id.to_string(),
        };
        if path.join(&to_file).exists() {
            return Err(Error::AlreadyExists(path.display().to_string()));
        }
        fs::create_dir_all(&path).await?;
        info!("create {}", path.display());
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.join("poster.jpg"))
            .await?
            .write_all(&self.poster)
            .await?;
        info!("write poster.jpg to {}", path.join("poster.jpg").display());
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.join("fanart.jpg"))
            .await?
            .write_all(&self.fanart)
            .await?;
        info!("write fanart.jpg to {}", path.join("fanart.jpg").display());
        let nfo = self.to_nfo()?;
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.join(MOVIE_NFO))
            .await?
            .write_all(nfo.as_bytes())
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

    pub fn check(self) -> Option<Info> {
        if self.title.is_empty() {
            return None;
        }
        if self.plot.is_empty() {
            return None;
        }
        if self.runtime == 0 {
            return None;
        }
        if self.id.is_empty() {
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

    fn combine_vec<T: Eq + Hash>(left: Vec<T>, right: Vec<T>) -> Vec<T> {
        let mut hset = HashSet::new();
        for item in left {
            hset.insert(item);
        }
        for item in right {
            hset.insert(item);
        }

        hset.into_iter().collect()
    }

    fn select_long(left: String, right: String) -> String {
        if left.len() > right.len() {
            left
        } else {
            right
        }
    }

    #[cfg(debug_assertions)]
    fn show_info(&self) {
        info!("title: {}", self.title);
        info!("rating: {}", self.rating);
        info!("plot: {}", self.plot);
        info!("runtime: {}", self.runtime);
        info!("id: {}", self.id);
        info!("genres: {:#?}", self.genres);
        info!("director: {}", self.director);
        info!("premiered: {}", self.premiered);
        info!("studio: {}", self.studio);
        info!("actors: {:#?}", self.actors);
        info!(
            "poster: {}",
            if self.poster.is_empty() { "no" } else { "yes" }
        );
        info!(
            "fanart: {}",
            if self.fanart.is_empty() { "no" } else { "yes" }
        );
    }

    pub fn merge(mut self, other: Info) -> Info {
        #[cfg(debug_assertions)]
        {
            info!("{:-^25}", " DEBUG INFO BEGIN ");
            self.show_info();
            info!("{:-^20}", " AFTER ");
            other.show_info();
            info!("{:-^25}", " DEBUG INFO END ");
        }

        if self.title.is_empty() {
            self.title = Info::select_long(self.title, other.title);
        }
        if self.rating == 0.0 {
            self.rating = other.rating;
        }
        if self.plot.is_empty() {
            self.plot = Info::select_long(self.plot, other.plot);
        }
        if self.runtime == 0 {
            self.runtime = other.runtime;
        }
        self.genres = Info::combine_vec(self.genres, other.genres);
        if self.director.is_empty() {
            self.director = Info::select_long(self.director, other.director);
        }
        if self.premiered.is_empty() {
            self.premiered = Info::select_long(self.premiered, other.premiered);
        }
        if self.studio.is_empty() {
            self.studio = Info::select_long(self.studio, other.studio);
        }
        self.actors = Info::combine_vec(self.actors, other.actors);
        if self.poster.is_empty() {
            self.poster = other.poster;
        }
        if self.fanart.is_empty() {
            self.fanart = other.fanart;
        }

        self
    }

    pub fn title(mut self, title: String) -> Info {
        self.title = title;
        self
    }

    pub fn poster(mut self, poster: Vec<u8>) -> Info {
        self.poster = poster;
        self
    }

    pub fn fanart(mut self, fanart: Vec<u8>) -> Info {
        self.fanart = fanart;
        self
    }

    pub fn rating(mut self, rating: f64) -> Info {
        self.rating = rating;
        self
    }

    pub fn plot(mut self, plot: String) -> Info {
        self.plot = plot;
        self
    }

    pub fn runtime(mut self, runtime: u32) -> Info {
        self.runtime = runtime;
        self
    }

    pub fn id(mut self, id: String) -> Info {
        self.id = id;
        self
    }

    pub fn genres(mut self, genres: Vec<String>) -> Info {
        self.genres = genres;
        self
    }

    pub fn director(mut self, director: String) -> Info {
        self.director = director;
        self
    }

    pub fn premiered(mut self, premiered: String) -> Info {
        self.premiered = premiered;
        self
    }

    pub fn studio(mut self, studio: String) -> Info {
        self.studio = studio;
        self
    }

    pub fn actors(mut self, actors: Vec<String>) -> Info {
        self.actors = actors;
        self
    }
}
