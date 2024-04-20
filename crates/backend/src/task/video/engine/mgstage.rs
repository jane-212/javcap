use std::sync::Arc;

use async_trait::async_trait;
use macros::Engine;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use scraper::Html;

use crate::select;
use crate::task::video::{Engine, Info, VideoParser};

#[derive(Engine)]
#[engine(image_loader)]
pub struct Mgstage {
    client: Arc<Client>,
    headers: HeaderMap,
}

impl Mgstage {
    const HOST: &'static str = "https://www.mgstage.com";

    pub fn new(client: Arc<Client>) -> Mgstage {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("www.mgstage.com"));
        headers.insert(header::REFERER, HeaderValue::from_static("https://www.mgstage.com/search/cSearch.php?search_word=abf-473&x=0&y=0&search_shop_id=&type=top"));
        headers.insert(header::COOKIE, HeaderValue::from_static("_ga_92ER0V7HV2=GS1.2.1713183847.3.1.1713185121.60.0.0; _ga_XGPRLSR61S=GS1.2.1713183847.3.1.1713185121.60.0.0; PHPSESSID=pjsi8cevfvq22doki1ah0qe8t5; uuid=fddc7ca043a1eaa16e98b08a3695fc0f; _ga=GA1.2.345338202.1712671719; _gat_UA-158726521-1=1; _gat_UA-58252858-1=1; _gid=GA1.2.722814334.1713101225; bWdzdGFnZS5jb20%3D-_lr_hb_-r2icil%2Fmgs={%22heartbeat%22:1713185120756}; bWdzdGFnZS5jb20%3D-_lr_tabs_-r2icil%2Fmgs={%22sessionID%22:0%2C%22recordingID%22:%225-817626ae-8e33-48da-833c-4f7e89aefd35%22%2C%22webViewID%22:null%2C%22lastActivity%22:1713185120756}; bWdzdGFnZS5jb20%3D-_lr_uf_-r2icil=45b577b4-7d76-4877-a776-f3f0bc66f98e; __ulfpc=202404142127045773; adc=1; coc=1"));

        Mgstage { client, headers }
    }

    async fn find_item(&self, video: &VideoParser) -> anyhow::Result<Option<String>> {
        select!(
            href: "#center_column > div.search_list > div > ul > li > h5 > a"
        );
        let url = format!("https://www.mgstage.com/search/cSearch.php?search_word={}&x=0&y=0&search_shop_id=&type=top", video.id());
        let res = self
            .client
            .get(url)
            .headers(self.headers.clone())
            .send()
            .await?
            .text()
            .await?;
        let doc = Html::parse_document(&res);
        let Some(href) = doc
            .select(&selectors().href)
            .next()
            .and_then(|href| href.attr("href"))
        else {
            return Ok(None);
        };

        Ok(Some(format!("{}{}", Mgstage::HOST, href)))
    }

    async fn load_info(&self, href: &str, info: &mut Info) -> anyhow::Result<Option<String>> {
        select!(
            title: "#center_column > div.common_detail_cover > h1",
            poster: "#center_column > div.common_detail_cover > div.detail_left > div > div > h2 > img",
            plot: "#introduction > dd > p.txt.introduction",
            tag: "#center_column > div.common_detail_cover > div.detail_left > div > table:nth-child(3) > tbody > tr"
        );
        let res = self
            .client
            .get(href)
            .headers(self.headers.clone())
            .send()
            .await?
            .text()
            .await?;
        let doc = Html::parse_document(&res);
        if let Some(title) = doc
            .select(&selectors().title)
            .next()
            .map(|title| title.inner_html().trim().to_string())
        {
            info.title(title);
        }
        let poster = doc
            .select(&selectors().poster)
            .next()
            .and_then(|img| img.attr("src").map(|src| src.to_string()));
        if let Some(plot) = doc
            .select(&selectors().plot)
            .next()
            .map(|plot| plot.inner_html().trim().to_string())
        {
            info.plot(plot);
        }
        let tags = doc
            .select(&selectors().tag)
            .map(|tag| tag.text().flat_map(|tag| tag.chars()).collect::<String>())
            .collect::<Vec<String>>();
        let tags = Mgstage::parse_tags(&tags);
        for (k, v) in tags {
            match k {
                "出演" => info.actors(vec![v.to_string()]),
                "収録時間" => info.runtime(
                    v.chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                        .unwrap_or(0),
                ),
                "配信開始日" => info.premiered(v.to_string()),
                "ジャンル" => info.genres(
                    v.lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect(),
                ),
                "評価" => info.rating(
                    v.chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect::<String>()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ),
                _ => {}
            }
        }

        Ok(poster)
    }

    fn parse_tags(tags: &[String]) -> Vec<(&str, &str)> {
        tags.iter()
            .flat_map(|tag| tag.split_once('：').map(|(k, v)| (k.trim(), v.trim())))
            .collect()
    }
}

#[async_trait]
impl Engine for Mgstage {
    async fn search(&self, video: &VideoParser) -> anyhow::Result<Info> {
        let mut info = Info::default();
        let Some(href) = self.find_item(video).await? else {
            return Ok(info);
        };
        if let Some(poster) = self.load_info(&href, &mut info).await? {
            let poster = self.load_img(&poster).await?;
            info.poster(poster);
        }

        Ok(info)
    }

    fn could_solve(&self, video: &VideoParser) -> bool {
        match video {
            VideoParser::FC2(_, _, _) => false,
            VideoParser::Normal(_, _, _) => true,
        }
    }

    fn id(&self) -> &'static str {
        self.key()
    }
}
