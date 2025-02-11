use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use scraper::{Html, Selector};
use video::VideoType;

use super::{select, Finder};

const HOST: &str = app::url::AVSOX;

select!(
    home_title: "#waterfall > div > a > div.photo-frame > img"
    home_date: "#waterfall > div > a > div.photo-info > span > date:nth-child(4)"
    home_url: "#waterfall > div > a"
    detail_fanart: "body > div.container > div.row.movie > div.col-md-9.screencap > a > img"
    detail_genre: "body > div.container > div.row.movie > div.col-md-3.info > p:nth-child(7) > span.genre > a"
    detail_info: "body > div.container > div.row.movie > div.col-md-3.info > p"
);

pub struct Avsox {
    base_url: String,
    client: Client,
    selectors: Selectors,
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
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let selectors = Selectors::new().with_context(|| "build selectors")?;
        let avsox = Avsox {
            base_url: base_url.unwrap_or(HOST.to_string()),
            client,
            selectors,
        };

        Ok(avsox)
    }
}

impl Display for Avsox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "avsox")
    }
}

#[async_trait]
impl Finder for Avsox {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => true,
            VideoType::Fc2(_) => true,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder()
            .id(key)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        let (url, poster) = self
            .find_in_home(key, &mut nfo)
            .await
            .with_context(|| "find in home")?;
        if let Some(poster) = poster {
            let poster = self
                .client
                .wait()
                .await
                .get(&poster)
                .send()
                .await?
                .bytes()
                .await?
                .to_vec();
            nfo.set_poster(poster);
        }

        if let Some(url) = url {
            let fanart = self
                .find_detail(&url, &mut nfo)
                .await
                .with_context(|| "find detail")?;
            if let Some(fanart) = fanart {
                let fanart = self
                    .client
                    .wait()
                    .await
                    .get(&fanart)
                    .send()
                    .await?
                    .bytes()
                    .await?
                    .to_vec();
                nfo.set_fanart(fanart);
            }
        }

        info!("{nfo:?}");
        Ok(nfo)
    }
}

impl Avsox {
    async fn find_in_home(
        &self,
        key: &VideoType,
        nfo: &mut Nfo,
    ) -> Result<(Option<String>, Option<String>)> {
        let url = format!("{}/cn/search/{key}", self.base_url);
        let text = self
            .client
            .wait()
            .await
            .get(&url)
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        if let Some(title) = html
            .select(&self.selectors.home_title)
            .next()
            .and_then(|node| node.attr("title"))
        {
            nfo.set_title(title.to_string());
        }

        let poster = html
            .select(&self.selectors.home_title)
            .next()
            .and_then(|node| node.attr("src").map(String::from));

        if let Some(date) = html
            .select(&self.selectors.home_date)
            .next()
            .map(|node| node.text().collect())
        {
            nfo.set_premiered(date);
        }

        let url = html
            .select(&self.selectors.home_url)
            .next()
            .and_then(|node| node.attr("href").map(|href| format!("https:{href}")));

        Ok((url, poster))
    }

    async fn find_detail(&self, url: &str, nfo: &mut Nfo) -> Result<Option<String>> {
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        let fanart = html
            .select(&self.selectors.detail_fanart)
            .next()
            .and_then(|node| node.attr("src").map(|src| src.to_string()));

        for genre in html.select(&self.selectors.detail_genre) {
            let genre = genre.text().collect();
            nfo.genres_mut().insert(genre);
        }

        let mut pairs = Vec::new();
        let mut prefix = "".to_string();
        for item in html.select(&self.selectors.detail_info) {
            let text = item.text().collect::<String>();
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

        Ok(fanart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn finder() -> Result<Avsox> {
        Avsox::builder().timeout(Duration::from_secs(5)).build()
    }

    #[test]
    fn test_support() -> Result<()> {
        let finder = finder()?;
        let videos = [
            VideoType::Jav("STARS".to_string(), "804".to_string()),
            VideoType::Fc2("3061625".to_string()),
        ];
        for video in videos {
            assert!(finder.support(&video));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find() -> Result<()> {
        let finder = finder()?;
        let cases = [
            (VideoType::Jav("HEYZO".to_string(), "3525".to_string()), {
                let mut nfo = Nfo::builder()
                    .id("HEYZO-3525")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build();
                nfo.set_title(
                    "竹田紀子 【たけだのりこ】 Sな淫乱痴熟女とねっとりエッチVol.2".to_string(),
                )
                .set_runtime(60)
                .set_studio("HEYZO".to_string())
                .set_premiered("2025-02-09".to_string());
                let genres = [
                    "舔阴",
                    "内射",
                    "手淫",
                    "第一视角",
                    "骑乘位",
                    "指法",
                    "痴女",
                    "后入",
                ];

                for genre in genres {
                    nfo.genres_mut().insert(genre.to_string());
                }
                nfo
            }),
            (VideoType::Fc2("1292936".to_string()), {
                let mut nfo = Nfo::builder()
                    .id("FC2-PPV-1292936")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build();
                nfo.set_title("【個人撮影・セット販売】妖艶から淫靡な妻へ 完全版".to_string())
                    .set_director("啼きの人妻".to_string())
                    .set_studio("FC2-PPV".to_string())
                    .set_runtime(68)
                    .set_premiered("2020-03-04".to_string());

                nfo
            }),
        ];
        for (video, expected) in cases {
            let actual = finder.find(&video).await?;
            assert!(!actual.fanart().is_empty());
            assert!(!actual.poster().is_empty());
            assert!(actual.subtitle().is_empty());
            assert_eq!(actual, expected);
        }

        Ok(())
    }
}
