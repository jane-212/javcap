use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::Nfo;
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use video::VideoType;

use super::Finder;

pub struct Javdb {
    base_url: String,
    client: Client,
}

#[bon]
impl Javdb {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Javdb> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(2)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;

        let javdb = Javdb {
            base_url: base_url.unwrap_or("https://javdb.com".to_string()),
            client,
        };
        Ok(javdb)
    }
}

#[async_trait]
impl Finder for Javdb {
    fn name(&self) -> &'static str {
        "javdb"
    }

    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => true,
            VideoType::Fc2(_) => false,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let name = key.name();
        let mut nfo = Nfo::new(&name)
            .with_country("日本".to_string())
            .with_mpaa("NC-17".to_string());

        let url = format!("{}/search", self.base_url);
        let text = self
            .client
            .wait()
            .await
            .get(&url)
            .query(&[("q", name.as_str()), ("f", "all")])
            .send()
            .await
            .with_context(|| format!("send to {url}"))?
            .text()
            .await
            .with_context(|| format!("decode to text from {url}"))?;
        let url = {
            let html = Document::from(text.as_str());

            let mut found = None;
            for item in html.find(Name("div").and(Class("item"))) {
                let Some(a) = item.find(Name("a").and(Class("box"))).next() else {
                    continue;
                };

                if a.find(Name("div").and(Class("video-title")).child(Name("strong")))
                    .next()
                    .map(|node| node.text() != name)
                    .unwrap_or(true)
                {
                    continue;
                }

                if let Some(title) = a.attr("title") {
                    nfo.set_title(title.to_string());
                }

                found = a
                    .attr("href")
                    .map(|href| format!("{}{href}", self.base_url));

                if let Some(score) = a
                    .find(
                        Name("div")
                            .and(Class("score"))
                            .child(Name("span").and(Class("value"))),
                    )
                    .next()
                    .and_then(|node| node.last_child())
                    .map(|node| node.text())
                {
                    let score = score.trim();
                    let score: f64 = score
                        .chars()
                        .filter(|c| *c == '.' || c.is_ascii_digit())
                        .collect::<String>()
                        .parse()
                        .unwrap_or_default();
                    nfo.set_rating(score * 2.0);
                }

                if let Some(meta) = a
                    .find(Name("div").and(Class("meta")))
                    .next()
                    .map(|node| node.text())
                {
                    let meta = meta.trim();
                    nfo.set_premiered(meta.to_string());
                }
            }

            found
        };

        if let Some(url) = url {
            let text = self
                .client
                .wait()
                .await
                .get(url)
                .send()
                .await?
                .text()
                .await?;
            {
                let html = Document::from(text.as_str());

                for block in html.find(Name("div").and(Class("panel-block"))) {
                    let Some(name) = block.find(Name("strong")).next().map(|node| node.text())
                    else {
                        continue;
                    };
                    let Some(value) = block
                        .find(Name("span").and(Class("value")))
                        .next()
                        .map(|node| node.text())
                    else {
                        continue;
                    };
                    let name = name.trim_end_matches(":").trim();
                    let value = value.trim();

                    match name {
                        "時長" => {
                            let runtime: u32 = value
                                .chars()
                                .filter(|c| c.is_ascii_digit())
                                .collect::<String>()
                                .parse()
                                .unwrap_or_default();
                            nfo.set_runtime(runtime);
                        }
                        "導演" => {
                            nfo.set_director(value.to_string());
                        }
                        "片商" => {
                            nfo.set_studio(value.to_string());
                        }
                        "類別" => {
                            let genres = value.split(",").collect::<Vec<_>>();
                            for genre in genres {
                                nfo.genres_mut().insert(genre.trim().to_string());
                            }
                        }
                        "演員" => {
                            let actors = value
                                .lines()
                                .map(|line| line.trim().trim_end_matches(['♂', '♀']))
                                .collect::<Vec<_>>();
                            for actor in actors {
                                nfo.actors_mut().insert(actor.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
