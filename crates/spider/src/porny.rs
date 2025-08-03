use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use scraper::Html;
use video::VideoType;

use super::{Finder, select};

const HOST: &str = "https://91porny.com";

select!(
    item: "#main > div.container-fluid.px-0 > div:nth-child(3) > div"
    title: "div > a.title.text-sub-title.mt-2.mb-1"
    author: "div > small > div:nth-child(1) > a"
    date: "div > small > div:nth-child(2)"
    runtime: "div > a.display.d-block > small"
    fanart: "div > a.display.d-block > div.img"
);

pub struct Porny {
    base_url: String,
    client: Client,
    selectors: Selectors,
}

#[bon]
impl Porny {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Porny> {
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

        let porny = Porny {
            base_url,
            selectors,
            client,
        };
        Ok(porny)
    }
}

impl Display for Porny {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "91 porny")
    }
}

#[async_trait]
impl Finder for Porny {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => false,
            VideoType::Fc2(_) => false,
            VideoType::Other(_) => true,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder()
            .id(key)
            .country(Country::China)
            .mpaa(Mpaa::NC17)
            .build();

        let fanart = self.search(key, &mut nfo).await?;
        let fanart = self
            .client
            .wait()
            .await
            .get(fanart)
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();
        nfo.set_fanart(fanart);
        nfo.set_studio("91".to_string());

        info!("{nfo:?}");
        Ok(nfo)
    }
}

impl Porny {
    async fn search(&self, key: &VideoType, nfo: &mut Nfo) -> Result<String> {
        let name = key.to_string();
        let url = format!("{}/search", self.base_url);
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("keywords", &name)])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        let Some(found) = html.select(&self.selectors.item).find(|item| {
            item.select(&self.selectors.title)
                .next()
                .map(|title| title.text().collect::<String>())
                .map(|title| title == name)
                .unwrap_or(false)
        }) else {
            bail!("item not found");
        };

        if let Some(title) = found
            .select(&self.selectors.title)
            .next()
            .map(|title| title.text().collect::<String>())
        {
            nfo.set_title(title);
        }

        if let Some(author) = found
            .select(&self.selectors.author)
            .next()
            .map(|author| author.text().collect::<String>())
        {
            nfo.set_director(author);
        }

        if let Some(date) = found
            .select(&self.selectors.date)
            .next()
            .map(|date| date.text().collect::<String>())
            .and_then(|date| {
                date.split_once('|')
                    .map(|(date, _)| date.trim().to_string())
            })
        {
            nfo.set_premiered(date);
        }

        if let Some(runtime) =
            found
                .select(&self.selectors.runtime)
                .next()
                .map(|runtime| runtime.text().collect::<String>())
                .map(|runtime| {
                    runtime.trim().split(':').take(2).enumerate().fold(
                        0,
                        |mut runtime, (idx, num)| {
                            let num = num.parse().unwrap_or(0);

                            match idx {
                                0 => {
                                    runtime += num * 60;
                                }
                                1 => {
                                    runtime += num;
                                }
                                _ => {}
                            }

                            runtime
                        },
                    )
                })
        {
            nfo.set_runtime(runtime);
        }

        found
            .select(&self.selectors.fanart)
            .next()
            .and_then(|img| img.attr("style"))
            .and_then(|sty| sty.split("'").nth(1).map(|fanart| fanart.to_string()))
            .ok_or(anyhow!("fanart not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn finder() -> Result<Porny> {
        Porny::builder().timeout(Duration::from_secs(10)).build()
    }

    #[test]
    fn test_support() -> Result<()> {
        let finder = finder()?;
        let videos = [
            (
                VideoType::Jav("STARS".to_string(), "804".to_string()),
                false,
            ),
            (VideoType::Fc2("3061625".to_string()), false),
            (VideoType::Other("hello".to_string()), true),
        ];
        for (video, supported) in videos {
            assert_eq!(finder.support(&video), supported);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find() -> Result<()> {
        let finder = finder()?;
        let cases = [(VideoType::Other("小飞棍来咯".to_string()), {
            let mut nfo = Nfo::builder()
                .id("小飞棍来咯")
                .country(Country::China)
                .mpaa(Mpaa::NC17)
                .build();
            nfo.set_premiered("2022-10-12".to_string());
            nfo.set_director("炮王大恶魔".to_string());
            nfo.set_title("小飞棍来咯".to_string());
            nfo.set_studio("91".to_string());
            nfo.set_runtime(3);

            nfo
        })];
        for (video, expected) in cases {
            let actual = finder.find(&video).await?;
            assert!(!actual.fanart().is_empty());
            assert!(actual.poster().is_empty());
            assert!(actual.subtitle().is_empty());
            assert_eq!(actual, expected);
        }

        Ok(())
    }
}
