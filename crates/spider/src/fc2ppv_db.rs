use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use video::VideoType;

use super::Finder;

pub struct Fc2ppvDB {
    client: Client,
}

impl Fc2ppvDB {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Fc2ppvDB> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(2)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;

        let fc2ppv_db = Fc2ppvDB { client };
        Ok(fc2ppv_db)
    }
}

#[async_trait]
impl Finder for Fc2ppvDB {
    fn name(&self) -> &'static str {
        "fc2ppv db"
    }

    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => false,
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

        let url = "https://fc2ppvdb.com/search";
        let name = match key {
            VideoType::Jav(id, key) => format!("{id}-{key}"),
            VideoType::Fc2(key) => key.clone(),
        };
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("stype", "title"), ("keyword", &name)])
            .send()
            .await
            .with_context(|| format!("send to {url}"))?
            .text()
            .await
            .with_context(|| format!("decode to text from {url}"))?;
        let img = {
            let html = Document::from(text.as_str());
            let Some(container) = html.find(Name("div").and(Class("container"))).next() else {
                bail!("container not found when find {name}");
            };
            let mut flexes = container.find(Name("div").and(Class("flex")));

            let meta = flexes.next();

            let mut img = None;
            if let Some(src) = meta
                .and_then(|node| node.find(Name("img")).next())
                .and_then(|img| img.attr("src"))
                .map(|src| src.to_string())
            {
                img = Some(src);
            }

            if let Some(info) =
                meta.and_then(|node| node.find(Name("div").and(Class("w-full"))).nth(1))
            {
                if let Some(title) = info
                    .find(Name("h2").and(Class("items-center")).child(Name("a")))
                    .next()
                    .map(|node| node.text())
                {
                    nfo.set_title(title);
                }

                for item in info.find(Name("div")) {
                    let text = item.text();
                    if let Some((name, value)) = text.split_once("：") {
                        let value = value.trim();
                        match name.trim() {
                            "販売者" => {
                                nfo.set_director(value.to_string());
                            }
                            "女優" => {
                                nfo.actors_mut().insert(value.to_string());
                            }
                            "販売日" => {
                                nfo.set_premiered(value.to_string());
                            }
                            "収録時間" => {
                                let mut h = 0;
                                let mut m = 0;
                                let mut s = 0;
                                for v in value.split(":").collect::<Vec<_>>().into_iter().rev() {
                                    let v = v.trim();
                                    if s == 0 {
                                        s = v.parse().unwrap_or_default();
                                        continue;
                                    }

                                    if m == 0 {
                                        m = v.parse().unwrap_or_default();
                                        continue;
                                    }

                                    if h == 0 {
                                        h = v.parse().unwrap_or_default();
                                    }
                                }
                                let runtime = h * 60 + m;
                                nfo.set_runtime(runtime);
                            }
                            "タグ" => {
                                for line in value.lines() {
                                    nfo.genres_mut().insert(line.trim().to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            if let Some(rating) = flexes
                .next()
                .and_then(|node| node.find(Name("span").and(Attr("id", "percentage"))).next())
                .map(|node| node.text())
            {
                let rating = rating
                    .trim()
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .map(|rating: f64| rating / 10.0)
                    .unwrap_or_default();
                nfo.set_rating(rating);
            }

            img
        };
        if let Some(url) = img {
            let img = self
                .client
                .wait()
                .await
                .get(&url)
                .send()
                .await
                .with_context(|| format!("send to {url}"))?
                .bytes()
                .await
                .with_context(|| format!("decode to bytes from {url}"))?
                .to_vec();
            nfo.set_fanart(img.clone());
            nfo.set_poster(img);
        }

        nfo.set_studio("FC2-PPV".to_string());

        info!("{}", nfo.summary());
        Ok(nfo)
    }
}
