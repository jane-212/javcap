use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use scraper::Html;
use video::VideoType;

use super::{Finder, select};

const HOST: &str = "https://fc2ppvdb.com";

select!(
    img: "body > div > div > div > main > div > section > div.container.lg\\:px-5.px-2.py-12.mx-auto > div.flex.flex-col.items-start.rounded-lg.shadow.md\\:flex-row.dark\\:border-gray-800.dark\\:bg-gray-900.py-2 > div.lg\\:w-2\\/5.w-full.mb-12.md\\:mb-0 > a > img"
    rating: "#percentage"
    title: "body > div > div > div > main > div > section > div.container.lg\\:px-5.px-2.py-12.mx-auto > div.flex.flex-col.items-start.rounded-lg.shadow.md\\:flex-row.dark\\:border-gray-800.dark\\:bg-gray-900.py-2 > div.w-full.lg\\:pl-8.px-2.lg\\:w-3\\/5 > h2 > a"
    item: "body > div > div > div > main > div > section > div.container.lg\\:px-5.px-2.py-12.mx-auto > div.flex.flex-col.items-start.rounded-lg.shadow.md\\:flex-row.dark\\:border-gray-800.dark\\:bg-gray-900.py-2 > div.w-full.lg\\:pl-8.px-2.lg\\:w-3\\/5 > div"
);

pub struct Fc2ppvDB {
    base_url: String,
    client: Client,
    selectors: Selectors,
}

#[bon]
impl Fc2ppvDB {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Fc2ppvDB> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let selectors = Selectors::new().with_context(|| "build selectors")?;
        let base_url = match base_url {
            Some(url) => url,
            None => String::from(HOST),
        };

        let fc2ppv_db = Fc2ppvDB {
            base_url,
            client,
            selectors,
        };
        Ok(fc2ppv_db)
    }
}

impl Display for Fc2ppvDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fc2ppv db")
    }
}

#[async_trait]
impl Finder for Fc2ppvDB {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => false,
            VideoType::Fc2(_) => true,
            VideoType::Other(_) => false,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder()
            .id(key)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        let img = self
            .find_detail(key, &mut nfo)
            .await
            .with_context(|| "find detail")?;
        let img = self
            .client
            .wait()
            .await
            .get(img)
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();
        nfo.set_fanart(img.clone());
        nfo.set_poster(img);

        info!("{nfo:?}");
        Ok(nfo)
    }
}

impl Fc2ppvDB {
    async fn find_detail(&self, key: &VideoType, nfo: &mut Nfo) -> Result<String> {
        let url = format!("{}/search", self.base_url);
        let name = match key {
            VideoType::Jav(id, number) => format!("{id}-{number}"),
            VideoType::Fc2(number) => number.clone(),
            VideoType::Other(title) => title.clone(),
        };
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("stype", "title"), ("keyword", &name)])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        if let Some(rating) = html
            .select(&self.selectors.rating)
            .next()
            .map(|node| node.text().collect::<String>())
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

        if let Some(title) = html
            .select(&self.selectors.title)
            .next()
            .map(|node| node.text().collect::<String>())
        {
            nfo.set_title(title);
        }

        for item in html.select(&self.selectors.item) {
            let text = item.text().collect::<String>();
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
                        for (idx, v) in value
                            .split(":")
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .skip(1)
                            .take(3)
                            .enumerate()
                        {
                            let v = v.trim();
                            if idx == 0 {
                                m = v.parse().unwrap_or_default();
                            }
                            if idx == 1 {
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

        html.select(&self.selectors.img)
            .next()
            .and_then(|node| node.attr("src").map(|src| src.to_string()))
            .ok_or_else(|| anyhow!("img not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn finder() -> Result<Fc2ppvDB> {
        Fc2ppvDB::builder().timeout(Duration::from_secs(10)).build()
    }

    #[test]
    fn test_support() -> Result<()> {
        let finder = finder()?;
        let videos = [
            (
                VideoType::Jav("STARS".to_string(), "804".to_string()),
                false,
            ),
            (VideoType::Fc2("3061625".to_string()), true),
        ];
        for (video, supported) in videos {
            assert_eq!(finder.support(&video), supported);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find() -> Result<()> {
        let finder = finder()?;
        let cases = [
            (VideoType::Fc2("3061625".to_string()), {
                let mut nfo = Nfo::builder()
                    .id("FC2-PPV-3061625")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build();
                nfo.set_title(
                "人生初めてのハメ撮り。そして中出し。学年一の美●女が覆面男の精子に汚される瞬間！"
                    .to_string(),
            )
            .set_director("KING POWER D".to_string())
            .set_runtime(82)
            .set_rating(9.2)
            .set_premiered("2022-07-30".to_string());
                nfo.actors_mut().insert("あすか".to_string());

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
                    .set_runtime(68)
                    .set_premiered("2020-03-04".to_string());
                nfo.actors_mut().insert("夏原あかり".to_string());

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
