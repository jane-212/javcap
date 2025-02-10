use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use video::VideoType;

use super::Finder;

pub struct Jav321 {
    client: Client,
}

impl Jav321 {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Jav321> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;

        let jav321 = Jav321 { client };
        Ok(jav321)
    }
}

impl Display for Jav321 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "jav321")
    }
}

#[async_trait]
impl Finder for Jav321 {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => true,
            VideoType::Fc2(_) => false,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder()
            .id(key)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        let url = "https://www.jav321.com/search";
        let text = self
            .client
            .wait()
            .await
            .post(url)
            .form(&[("sn", key.to_string())])
            .send()
            .await
            .with_context(|| format!("send to {url}"))?
            .text()
            .await
            .with_context(|| format!("decode to text from {url}"))?;
        let (fanart, poster) = {
            let html = Document::from(text.as_str());
            let Some(panel) = html.find(Name("div").and(Class("panel"))).next() else {
                bail!("panel not found");
            };

            if let Some(title) = panel
                .find(Name("div").and(Class("panel-heading")).child(Name("h3")))
                .next()
                .and_then(|heading| heading.first_child().map(|child| child.text()))
            {
                nfo.set_title(title.trim().to_string());
            }

            let mut poster = None;
            if let Some(body) = panel.find(Name("div").and(Class("panel-body"))).next() {
                if let Some(plot) = body
                    .last_child()
                    .and_then(|child| child.first_child())
                    .and_then(|child| child.first_child())
                    .map(|node| node.text())
                {
                    nfo.set_plot(plot);
                }

                poster = body
                    .first_child()
                    .and_then(|node| node.first_child())
                    .and_then(|node| node.first_child())
                    .and_then(|node| node.attr("src"))
                    .map(|src| src.to_string());

                if let Some(info) = body.first_child().and_then(|child| child.last_child()) {
                    let mut s = Vec::new();
                    for child in info.children() {
                        let text = child.text();
                        if text.starts_with(":") {
                            s.push(":".to_string());
                            s.push(text.trim_start_matches(":").trim().to_string());
                        } else {
                            s.push(text.trim().to_string());
                        }
                    }
                    let mut v = Vec::new();
                    while let Some(text) = s.pop() {
                        if text.is_empty() {
                            continue;
                        }

                        if text != ":" {
                            v.push(text);
                            continue;
                        }

                        if let Some(name) = s.pop() {
                            match name.as_str() {
                                "メーカー" => {
                                    if let Some(studio) = v.first() {
                                        nfo.set_studio(studio.to_string());
                                    }
                                }
                                "出演者" => {
                                    for actor in v.iter() {
                                        nfo.actors_mut().insert(actor.to_string());
                                    }
                                }
                                "ジャンル" => {
                                    for genre in v.iter() {
                                        nfo.genres_mut().insert(genre.to_string());
                                    }
                                }
                                "配信開始日" => {
                                    if let Some(date) = v.first() {
                                        nfo.set_premiered(date.to_string());
                                    }
                                }
                                "収録時間" => {
                                    if let Some(runtime) = v.first() {
                                        let runtime: String = runtime
                                            .chars()
                                            .filter(|c| c.is_ascii_digit())
                                            .collect();
                                        let runtime: u32 = runtime.parse().unwrap_or_default();
                                        nfo.set_runtime(runtime);
                                    }
                                }
                                "平均評価" => {
                                    if let Some(rating) = v.first() {
                                        let rating: f64 = rating.parse().unwrap_or_default();
                                        nfo.set_rating(rating);
                                    }
                                }
                                _ => {}
                            }
                        }

                        v.clear();
                    }
                }
            }

            let mut fanart = None;
            if let Some(items) = html.find(Name("div").and(Class("col-md-3"))).last() {
                if let Some(src) = items
                    .first_child()
                    .and_then(|child| child.first_child())
                    .and_then(|child| child.first_child())
                    .and_then(|child| child.first_child())
                    .and_then(|node| node.attr("src"))
                {
                    fanart = Some(src.to_string());
                }
            }

            (fanart, poster)
        };
        if let Some(fanart) = fanart {
            let fanart = self
                .client
                .wait()
                .await
                .get(&fanart)
                .send()
                .await
                .with_context(|| format!("send to {fanart}"))?
                .bytes()
                .await
                .with_context(|| format!("decode to bytes from {fanart}"))?;
            nfo.set_fanart(fanart.to_vec());
        }
        if let Some(poster) = poster {
            let poster = self
                .client
                .wait()
                .await
                .get(&poster)
                .send()
                .await
                .with_context(|| format!("send to {poster}"))?
                .bytes()
                .await
                .with_context(|| format!("decode to bytes from {poster}"))?;
            nfo.set_poster(poster.to_vec());
        }

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
