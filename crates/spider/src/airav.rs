use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use scraper::{Html, Selector};
use video::VideoType;

use super::{select, Finder};

const HOST: &str = app::url::AIRAV;

select!(
    home_title: "body > div:nth-child(4) > div > div.row.row-cols-2.row-cols-lg-4.g-2.mt-0 > div:nth-child(1) > div > div.oneVideo-body > h5"
    home_fanart: "body > div:nth-child(4) > div > div.row.row-cols-2.row-cols-lg-4.g-2.mt-0 > div:nth-child(1) > div > div.oneVideo-top > a > img"
    home_url: "body > div:nth-child(4) > div > div.row.row-cols-2.row-cols-lg-4.g-2.mt-0 > div:nth-child(1) > div > div.oneVideo-top > a"
    detail_date: "body > div:nth-child(4) > div.container > div > div.col-lg-9.col-12.pt-3 > div.video-item > div.me-4"
    detail_plot: "body > div:nth-child(4) > div.container > div > div.col-lg-9.col-12.pt-3 > div.video-info > p"
    detail_name: "body > div:nth-child(4) > div.container > div > div.col-lg-9.col-12.pt-3 > div.video-info > div > ul > li"
    detail_tag: "a"
);

pub struct Airav {
    client: Client,
    selectors: Selectors,
}

impl Airav {
    pub fn new(timeout: Duration, proxy: Option<String>) -> Result<Airav> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let selectors = Selectors::new().with_context(|| "build selectors")?;

        let airav = Airav { client, selectors };
        Ok(airav)
    }
}

impl Display for Airav {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "airav")
    }
}

#[async_trait]
impl Finder for Airav {
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

        let (url, fanart) = self
            .find_in_home(key, &mut nfo)
            .await
            .with_context(|| "find in home")?;
        if let Some(fanart) = fanart {
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
        }
        if let Some(url) = url {
            self.find_detail(&url, &mut nfo)
                .await
                .with_context(|| "find detail")?;
        }

        info!("{nfo:?}");
        Ok(nfo)
    }
}

impl Airav {
    async fn find_in_home(
        &self,
        key: &VideoType,
        nfo: &mut Nfo,
    ) -> Result<(Option<String>, Option<String>)> {
        let url = format!("{HOST}/search_result");
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("kw", key.to_string())])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);

        if let Some(title) = html
            .select(&self.selectors.home_title)
            .next()
            .map(|node| node.text().collect())
        {
            nfo.set_title(title);
        }

        let fanart = html
            .select(&self.selectors.home_fanart)
            .next()
            .and_then(|node| node.attr("src").map(String::from));

        let url = html
            .select(&self.selectors.home_url)
            .next()
            .and_then(|node| node.attr("href").map(|href| format!("{HOST}{href}")));

        Ok((url, fanart))
    }

    async fn find_detail(&self, url: &str, nfo: &mut Nfo) -> Result<()> {
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

        if let Some(date) = html
            .select(&self.selectors.detail_date)
            .next()
            .and_then(|node| node.text().last())
            .and_then(|text| text.split_once(' ').map(|(date, _)| date))
            .map(String::from)
        {
            nfo.set_premiered(date);
        }

        if let Some(plot) = html
            .select(&self.selectors.detail_plot)
            .next()
            .map(|node| node.text().collect())
        {
            nfo.set_plot(plot);
        }

        for item in html.select(&self.selectors.detail_name) {
            let Some(name) = item.text().next() else {
                continue;
            };

            match name.trim().trim_end_matches('：') {
                "女優" => {
                    for tag in item.select(&self.selectors.detail_tag) {
                        let tag = tag.text().collect();
                        nfo.actors_mut().insert(tag);
                    }
                }
                "標籤" => {
                    for tag in item.select(&self.selectors.detail_tag) {
                        let tag = tag.text().collect();
                        nfo.genres_mut().insert(tag);
                    }
                }
                "廠商" => {
                    if let Some(tag) = item
                        .select(&self.selectors.detail_tag)
                        .next()
                        .map(|node| node.text().collect())
                    {
                        nfo.set_studio(tag);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn finder() -> Result<Airav> {
        Airav::new(Duration::from_secs(5), None)
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
            (VideoType::Jav("STARS".to_string(), "804".to_string()), {
                let mut nfo = Nfo::builder()
                    .id("STARS-804")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build();
                nfo.set_title("STARS-804 馬賽克破壞版 STARS-804 神木麗".to_string())
                    .set_plot("馬賽克破壞版 STARS-804 神木麗".to_string())
                    .set_premiered("2023-04-11".to_string());
                let actors = ["神木麗"];
                let genres = ["無碼", "馬賽克破壞版"];

                for actor in actors {
                    nfo.actors_mut().insert(actor.to_string());
                }
                for genre in genres {
                    nfo.genres_mut().insert(genre.to_string());
                }
                nfo
            }),
            (VideoType::Jav("IPX".to_string(), "443".to_string()), {
                let mut nfo = Nfo::builder()
                    .id("IPX-443")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build();
                nfo.set_title("IPX-443 禁欲一個月後與朋友男友瘋狂做愛 明里紬 合計10回密着性交".to_string())
                    .set_plot("雖然我有老婆，但是看到了老婆朋友明里紬誘惑後就心癢癢。知道老婆要不在家開始，就禁慾一個月，只為了與明里紬從早到晚瘋狂做愛…".to_string())
                    .set_studio("IDEA POCKET".to_string())
                    .set_premiered("2020-02-13".to_string());
                let actors = ["愛里留衣", "明里紬"];
                let genres = [
                    "中文",
                    "寝取り・寝取られ・ntr",
                    "拘束",
                    "紀錄片",
                    "中出",
                    "AV女優片",
                ];

                for actor in actors {
                    nfo.actors_mut().insert(actor.to_string());
                }
                for genre in genres {
                    nfo.genres_mut().insert(genre.to_string());
                }
                nfo
            }),
            (VideoType::Fc2("3061625".to_string()), {
                let mut nfo = Nfo::builder()
                    .id("FC2-PPV-3061625")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build();
                nfo.set_title(
                    "FC2-PPV-3061625 人生初拍攝。中出。年度最美少女被蒙面男子精子玷污的那一刻！"
                        .to_string(),
                )
                .set_plot("本站獨家FC2素人影片，千萬別錯過!!".to_string())
                .set_studio("FC2高清版".to_string())
                .set_premiered("2022-07-30".to_string());
                let genres = [
                    "素人",
                    "真實素人",
                    "fc2ppv",
                    "720p",
                    "自拍",
                    "巨乳",
                    "個人撮影",
                    "學生妹",
                ];

                for genre in genres {
                    nfo.genres_mut().insert(genre.to_string());
                }
                nfo
            }),
        ];
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
