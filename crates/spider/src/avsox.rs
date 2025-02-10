use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Name, Predicate};
use video::VideoType;

use super::Finder;

pub struct Avsox {
    base_url: String,
    client: Client,
}

#[bon]
impl Avsox {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Avsox> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(2)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let avsox = Avsox {
            base_url: base_url.unwrap_or("https://avsox.click".to_string()),
            client,
        };

        Ok(avsox)
    }

    fn find_item<'a>(html: &'a Document, name: &str) -> Option<Node<'a>> {
        html.find(
            Name("div")
                .and(Attr("id", "waterfall"))
                .child(Name("div").and(Class("item"))),
        )
        .find(|item| {
            item.find(
                Name("div")
                    .and(Class("photo-info"))
                    .descendant(Name("date")),
            )
            .next()
            .map(|date| date.text() == name)
            .unwrap_or(false)
        })
    }
}

#[async_trait]
impl Finder for Avsox {
    fn name(&self) -> &'static str {
        "avsox"
    }

    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => true,
            VideoType::Fc2(_) => true,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let name = key.name();
        let mut nfo = Nfo::builder()
            .id(&name)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        let url = format!("{}/cn/search/{name}", self.base_url);
        let text = self
            .client
            .wait()
            .await
            .get(&url)
            .send()
            .await
            .with_context(|| format!("send to {url}"))?
            .text()
            .await
            .with_context(|| format!("decode to text from {url}"))?;
        let (url, poster) = {
            let html = Document::from(text.as_str());

            let Some(item) = Self::find_item(&html, &name) else {
                bail!("{name} not found");
            };

            let mut poster = None;
            if let Some(img) = item
                .find(
                    Name("div")
                        .and(Class("photo-frame"))
                        .descendant(Name("img")),
                )
                .next()
            {
                if let Some(title) = img.attr("title") {
                    let title = title.trim().to_string();
                    nfo.set_title(title);
                }

                if let Some(src) = img.attr("src") {
                    poster = Some(src.to_string());
                }
            }

            if let Some(date) = item
                .find(
                    Name("div")
                        .and(Class("photo-info"))
                        .descendant(Name("date")),
                )
                .nth(1)
                .map(|date| date.text())
            {
                nfo.set_premiered(date.trim().to_string());
            }

            let mut url = None;
            if let Some(href) = item
                .find(Name("a").and(Class("movie-box")))
                .next()
                .and_then(|a| a.attr("href"))
            {
                url = Some(href.to_string());
            }

            (url, poster)
        };
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

        if let Some(url) = url {
            let url = format!("https:{url}");
            let text = self
                .client
                .wait()
                .await
                .get(&url)
                .send()
                .await
                .with_context(|| format!("send to {url}"))?
                .text()
                .await
                .with_context(|| format!("decode to text from {url}"))?;
            let fanart = {
                let html = Document::from(text.as_str());

                let Some(container) = html.find(Name("div").and(Class("container"))).nth(1) else {
                    bail!("container not found when find {name}");
                };

                let mut fanart = None;
                if let Some(href) = container
                    .find(Name("a").and(Class("bigImage")))
                    .next()
                    .and_then(|a| a.attr("href"))
                {
                    fanart = Some(href.to_string());
                }

                let mut pairs = Vec::new();
                let mut prefix = "".to_string();
                for item in container.find(Name("div").and(Class("col-md-3")).child(Name("p"))) {
                    let mut skip = false;
                    for genre in item.find(Name("span").and(Class("genre")).child(Name("a"))) {
                        let genre = genre.text();
                        nfo.genres_mut().insert(genre);
                        skip = true;
                    }
                    if skip {
                        continue;
                    }

                    let text = item.text();
                    let text = text.trim();

                    if !text.contains(":") {
                        pairs.push((prefix.clone(), text.to_string()));
                        continue;
                    }

                    if text.ends_with(":") {
                        prefix = text.trim_end_matches(":").to_string();
                        continue;
                    }

                    if let Some((name, value)) = text.split_once(":") {
                        pairs.push((name.trim().to_string(), value.trim().to_string()));
                    }
                }

                for pair in pairs {
                    match pair.0.as_str() {
                        "制作商" => {
                            nfo.set_studio(pair.1);
                        }
                        "系列" => {
                            nfo.set_director(pair.1);
                        }
                        "长度" => {
                            let number: String =
                                pair.1.chars().take_while(|c| c.is_ascii_digit()).collect();
                            let runtime: u32 = number.parse().unwrap_or_default();
                            nfo.set_runtime(runtime);
                        }
                        _ => {}
                    }
                }

                fanart
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
        }

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
