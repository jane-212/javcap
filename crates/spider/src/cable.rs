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

use super::{Finder, select, which_country};

const HOST: &str = "https://www.hsav.xyz";

select!(
    home_item: "#main-content > div > div.blog-items.blog-items-control.site__row.grid-default > article.post-item"
    home_title: "div > div.blog-pic > div > a > img"
    home_date: "div > div.listing-content > div.entry-meta.post-meta.meta-font > div > div.date-time > span > time"
);

pub struct Cable {
    base_url: String,
    client: Client,
    selectors: Selectors,
}

#[bon]
impl Cable {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Cable> {
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

        let cable = Cable {
            base_url,
            client,
            selectors,
        };
        Ok(cable)
    }
}

impl Display for Cable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cable")
    }
}

#[async_trait]
impl Finder for Cable {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => true,
            VideoType::Fc2(_) => true,
            VideoType::Other(_) => false,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder()
            .id(key)
            .country(which_country(key))
            .mpaa(Mpaa::NC17)
            .build();

        let img = self
            .find_home(key, &mut nfo)
            .await
            .with_context(|| "find home")?;
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
        if Country::China == *nfo.country() {
            nfo.set_poster(img.clone());
            nfo.set_fanart(img);
        } else {
            nfo.set_fanart(img);
        }

        info!("{nfo:?}");
        Ok(nfo)
    }
}

impl Cable {
    async fn find_home(&self, key: &VideoType, nfo: &mut Nfo) -> Result<String> {
        let url = format!("{}/index/data/search.html", self.base_url);
        let name = match &key {
            VideoType::Jav(id, number) => format!("{id}-{number}"),
            VideoType::Fc2(number) => format!("FC2PPV-{number}"),
            VideoType::Other(title) => title.clone(),
        };
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("k", &name)])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        let mut img = None;
        for item in html.select(&self.selectors.home_item) {
            let Some(title) = item
                .select(&self.selectors.home_title)
                .next()
                .and_then(|node| node.attr("alt"))
                .and_then(|title| {
                    if title.to_uppercase().contains(&name) {
                        Some(title)
                    } else {
                        None
                    }
                })
            else {
                continue;
            };
            nfo.set_title(title.to_string());

            img = item
                .select(&self.selectors.home_title)
                .next()
                .and_then(|node| node.attr("data-src").map(String::from));

            if let Some(date) = item
                .select(&self.selectors.home_date)
                .next()
                .and_then(|node| node.attr("datetime"))
                .and_then(|date| date.split_once(' ').map(|(date, _)| date.trim()))
            {
                nfo.set_premiered(date.to_string());
            }

            break;
        }

        img.ok_or_else(|| anyhow!("img not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn finder() -> Result<Cable> {
        Cable::builder().timeout(Duration::from_secs(10)).build()
    }

    #[test]
    fn test_support() -> Result<()> {
        let finder = finder()?;
        let videos = [
            (VideoType::Jav("STARS".to_string(), "804".to_string()), true),
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
            (VideoType::Jav("PRED".to_string(), "323".to_string()), {
                let mut nfo = Nfo::builder()
                    .id("PRED-323")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build();
                nfo.set_title("PRED-323性欲が強すぎる爆乳義姉と嫁の不在中にこっそり時短中出ししているオレ…JULIA".to_string())
                    .set_premiered("2024-10-20".to_string());

                nfo
            }),
            (VideoType::Fc2("4554988".to_string()), {
                let mut nfo = Nfo::builder()
                    .id("FC2-PPV-4554988")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build();
                nfo.set_title("FC2PPV-4554988-【無修正】＜美巨乳Ｆカップ＞出張メンエス嬢の身体が異常なエロさ！".to_string())
                    .set_premiered("2024-10-20".to_string());

                nfo
            }),
            (VideoType::Jav("MD".to_string(), "0331".to_string()), {
                let mut nfo = Nfo::builder()
                    .id("MD-0331")
                    .country(Country::China)
                    .mpaa(Mpaa::NC17)
                    .build();
                nfo.set_title(
                    "麻豆传媒映画.MD-0331.雯雯.我的房东是个萌妹子.处女催租肉体缴付".to_string(),
                )
                .set_premiered("2024-10-17".to_string());

                nfo
            }),
        ];
        for (video, expected) in cases {
            let actual = finder.find(&video).await?;
            assert!(!actual.fanart().is_empty());
            if Country::China == *actual.country() {
                assert!(!actual.poster().is_empty());
            } else {
                assert!(actual.poster().is_empty());
            }
            assert!(actual.subtitle().is_empty());
            assert_eq!(actual, expected);
        }

        Ok(())
    }
}
