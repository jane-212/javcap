use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use scraper::Html;
use video::VideoType;

use super::{select, Finder};

const HOST: &str = "https://airav.io";

select!(
    home_item: "body > div:nth-child(4) > div > div.row.row-cols-2.row-cols-lg-4.g-2.mt-0 > div"
    home_title: "div > div.oneVideo-body > h5"
    home_fanart: "div > div.oneVideo-top > a > img"
    home_url: "div > div.oneVideo-top > a"
    detail_date: "body > div:nth-child(4) > div.container > div > div.col-lg-9.col-12.pt-3 > div.video-item > div.me-4"
    detail_plot: "body > div:nth-child(4) > div.container > div > div.col-lg-9.col-12.pt-3 > div.video-info > p"
    detail_name: "body > div:nth-child(4) > div.container > div > div.col-lg-9.col-12.pt-3 > div.video-info > div > ul > li"
    detail_tag: "a"
);

pub struct Airav {
    base_url: String,
    client: Client,
    selectors: Selectors,
}

#[bon]
impl Airav {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Airav> {
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

        let airav = Airav {
            base_url,
            client,
            selectors,
        };
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
            VideoType::Other(_) => false,
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
        let url = format!("{}/search_result", self.base_url);
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

        let mut url = None;
        let mut fanart = None;
        let name = key.to_string();
        for item in html.select(&self.selectors.home_item) {
            let Some(title) = item
                .select(&self.selectors.home_title)
                .next()
                .map(|node| node.text().collect::<String>())
            else {
                continue;
            };
            if !title.contains(&name) || title.contains("克破") {
                continue;
            }
            nfo.set_title(title.trim_start_matches(&name).trim().to_string());

            fanart = item
                .select(&self.selectors.home_fanart)
                .next()
                .and_then(|node| node.attr("src").map(String::from));

            url = item
                .select(&self.selectors.home_url)
                .next()
                .and_then(|node| {
                    node.attr("href")
                        .map(|href| format!("{}{href}", self.base_url))
                });

            if url.is_some() && fanart.is_some() {
                break;
            }
        }

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
        Airav::builder().timeout(Duration::from_secs(5)).build()
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
                nfo.set_title("隨著本能絡合的極上內衣與精油4本番 神木麗".to_string())
                    .set_plot("G罩杯的身材，搭配高級內衣和按摩油更能突顯出其絕佳的比例。神木麗進入飯店後懇求般進行濃厚的性愛。綑縛美麗四肢，持續玩弄身軀到絕頂後懇求插入，喘息聲無法壓抑響徹於房間…".to_string())
                    .set_studio("SOD".to_string())
                    .set_premiered("2023-04-06".to_string());
                let actors = ["神木麗"];
                let genres = [
                    "巨乳",
                    "720p",
                    "HD高畫質",
                    "ローション・オイル",
                    "AV女優片",
                    "乳交",
                    "中文",
                    "性感內衣",
                ];

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
                nfo.set_title("禁欲一個月後與朋友男友瘋狂做愛 明里紬 合計10回密着性交".to_string())
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
                    "人生初拍攝。中出。年度最美少女被蒙面男子精子玷污的那一刻！".to_string(),
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
