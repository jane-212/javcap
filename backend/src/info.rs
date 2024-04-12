use error::{Error, Result};
use std::path::Path;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[derive(Default)]
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
}

impl ToString for Info {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl Info {
    pub fn new() -> Info {
        Info {
            mpaa: "NC-17".to_string(),
            country: "日本".to_string(),
            ..Default::default()
        }
    }

    pub async fn write_to(self, path: &Path) -> Result<()> {
        let path = path.join(&self.studio).join(&self.id);
        if path.exists() {
            return Err(Error::AlreadyExists(self.id));
        }
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .await?
            .write_all(self.to_string().as_bytes())
            .await?;

        Ok(())
    }

    pub fn check(self) -> Option<Info> {
        if self.title.is_empty() {
            return None;
        }
        if self.rating == 0.0 {
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

        Some(self)
    }

    pub fn merge(&mut self, other: Info) {
        if self.title.is_empty() {
            self.title = other.title;
        }
        if self.rating == 0.0 {
            self.rating = other.rating;
        }
        if self.plot.is_empty() {
            self.plot = other.plot;
        }
        if self.runtime == 0 {
            self.runtime = other.runtime;
        }
        if self.id.is_empty() {
            self.id = other.id;
        }
        if self.genres.is_empty() {
            self.genres = other.genres;
        }
        if self.director.is_empty() {
            self.director = other.director;
        }
        if self.premiered.is_empty() {
            self.premiered = other.premiered;
        }
        if self.studio.is_empty() {
            self.studio = other.studio;
        }
        if self.actors.is_empty() {
            self.actors = other.actors;
        }
    }

    pub fn title(mut self, title: String) -> Info {
        self.title = title;
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
