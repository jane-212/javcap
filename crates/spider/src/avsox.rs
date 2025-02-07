use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use nfo::Nfo;
use ratelimit::Ratelimiter;
use reqwest::{Client, Proxy};
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Name, Predicate};
use tokio::time;
use video::VideoType;

use super::Finder;

pub struct Avsox {
    base_url: String,
    limiter: Ratelimiter,
    client: Client,
}

impl Avsox {
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Avsox> {
        let limiter = Ratelimiter::builder(1, Duration::from_secs(2))
            .initial_available(1)
            .build()?;
        let mut client_builder = Client::builder()
            .timeout(timeout)
            .user_agent(app::USER_AGENT);
        if let Some(url) = proxy {
            let proxy = Proxy::https(url)?;
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder.build()?;
        let avsox = Avsox {
            base_url: base_url.unwrap_or("https://avsox.click".to_string()),
            client,
            limiter,
        };

        Ok(avsox)
    }

    async fn wait_limiter(&self) {
        loop {
            match self.limiter.try_wait() {
                Ok(_) => break,
                Err(sleep) => time::sleep(sleep).await,
            }
        }
    }

    fn find_item<'a>(html: &'a Document, key: &VideoType) -> Option<Node<'a>> {
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
            .map(|date| date.text() == key.name())
            .unwrap_or(false)
        })
    }
}

#[async_trait]
impl Finder for Avsox {
    async fn find(&self, key: VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::new(key.name());
        nfo.set_country("日本".to_string());
        nfo.set_mpaa("NC-17".to_string());

        let url = format!("{}/cn/search/{}", self.base_url, key.name());
        self.wait_limiter().await;
        let text = self.client.get(url).send().await?.text().await?;
        let (url, poster) = {
            let html = Document::from(text.as_str());

            let Some(item) = Self::find_item(&html, &key) else {
                return Ok(nfo);
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
                    nfo.set_title(title.clone());
                    nfo.set_plot(title);
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
            self.wait_limiter().await;
            let poster = self.client.get(poster).send().await?.bytes().await?;
            nfo.set_poster(poster.to_vec());
        }

        if let Some(url) = url {
            let url = format!("https:{url}");
            self.wait_limiter().await;
            let text = self.client.get(url).send().await?.text().await?;
            let fanart = {
                let html = Document::from(text.as_str());

                let Some(container) = html.find(Name("div").and(Class("container"))).nth(1) else {
                    return Ok(nfo);
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
                            if nfo.director().is_empty() {
                                nfo.set_director(pair.1.clone());
                            }
                            nfo.actors_mut().insert(pair.1.clone());
                            nfo.genres_mut().insert(pair.1);
                        }
                        "长度" => {
                            let number: String =
                                pair.1.chars().take_while(|c| c.is_ascii_digit()).collect();
                            let rumtime: u32 = number.parse().unwrap_or_default();
                            nfo.set_runtime(rumtime);
                        }
                        _ => {}
                    }
                }

                fanart
            };
            if let Some(fanart) = fanart {
                self.wait_limiter().await;
                let fanart = self.client.get(fanart).send().await?.bytes().await?;
                nfo.set_fanart(fanart.to_vec());
            }
        }

        nfo.set_rating(0.1);

        Ok(nfo)
    }
}
