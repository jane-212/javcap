use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use reqwest::header::{self, HeaderMap, HeaderValue};
use scraper::Html;
use serde::Deserialize;
use serde_json::Value;
use video::VideoType;

use super::{Finder, select, which_country};

const HOST: &str = "https://theporndb.net";
const API_HOST: &str = "https://api.theporndb.net";

select!(
    data: "#app"
);

pub struct ThePornDB {
    base_url: String,
    api_url: String,
    client: Client,
    selectors: Selectors,
}

#[bon]
impl ThePornDB {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        api_url: Option<String>,
        key: impl AsRef<str>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Self> {
        let headers = {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", key.as_ref()))?,
            );
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

            headers
        };
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .headers(headers)
            .build()
            .with_context(|| "build http client")?;
        let base_url = match base_url {
            Some(url) => url,
            None => String::from(HOST),
        };
        let api_url = match api_url {
            Some(url) => url,
            None => String::from(API_HOST),
        };
        let selectors = Selectors::new().with_context(|| "build selectors")?;

        let this = Self {
            base_url,
            api_url,
            client,
            selectors,
        };
        Ok(this)
    }
}

impl Display for ThePornDB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the porn db")
    }
}

#[async_trait]
impl Finder for ThePornDB {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => !matches!(which_country(key), Country::China),
            VideoType::Fc2(_) => false,
            VideoType::Other(_) => false,
        }
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder()
            .id(key)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        let link = self.find_by_key(key).await?;
        let uuid = self.get_uuid_by_link(link).await?;
        self.load_data_by_uuid(uuid, &mut nfo).await?;

        info!("{nfo:?}");
        Ok(nfo)
    }
}

impl ThePornDB {
    async fn load_data_by_uuid(&self, uuid: String, nfo: &mut Nfo) -> Result<()> {
        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Response {
            pub data: Data,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Data {
            pub id: Value,
            #[serde(rename = "_id")]
            pub id2: Value,
            pub title: String,
            #[serde(rename = "type")]
            pub type_field: Value,
            pub slug: Value,
            pub external_id: Value,
            pub description: Value,
            pub rating: Value,
            pub site_id: Value,
            pub date: String,
            pub url: Value,
            pub image: Value,
            pub back_image: Value,
            pub poster: Value,
            pub trailer: Value,
            pub duration: i64,
            pub format: Value,
            pub sku: Value,
            pub posters: Option<Posters>,
            pub background: Option<Background>,
            pub background_back: Value,
            pub created: Value,
            pub last_updated: Value,
            pub performers: Vec<Performer>,
            pub site: Site,
            pub tags: Vec<Value>,
            pub hashes: Value,
            pub markers: Vec<Value>,
            pub directors: Vec<Director>,
            pub scenes: Vec<Value>,
            pub movies: Vec<Value>,
            pub links: Vec<Value>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Posters {
            pub large: String,
            pub medium: Value,
            pub small: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Background {
            pub full: Value,
            pub large: String,
            pub medium: Value,
            pub small: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Performer {
            pub id: Value,
            #[serde(rename = "_id")]
            pub id2: Value,
            pub slug: Value,
            pub site_id: Value,
            pub name: String,
            pub bio: Value,
            pub is_parent: Value,
            pub extra: Value,
            pub image: Value,
            pub thumbnail: Value,
            pub face: Value,
            pub parent: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Site {
            pub uuid: Value,
            pub id: Value,
            pub parent_id: Value,
            pub network_id: Value,
            pub name: String,
            pub short_name: Value,
            pub url: Value,
            pub description: Value,
            pub rating: Value,
            pub logo: Value,
            pub favicon: Value,
            pub poster: Value,
            pub network: Value,
            pub parent: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Director {
            pub id: Value,
            pub name: String,
            pub slug: Value,
        }

        let url = format!("{}/jav/{}", self.api_url, uuid);
        let res = self
            .client
            .wait()
            .await
            .get(url)
            .send()
            .await?
            .json::<Response>()
            .await?;

        let data = res.data;
        nfo.set_title(data.title);
        nfo.set_premiered(data.date);
        nfo.set_runtime(data.duration as u32 / 60);
        for actor in data.performers {
            nfo.actors_mut().insert(actor.name);
        }
        nfo.set_studio(data.site.name);
        if let Some(director) = data.directors.first() {
            nfo.set_director(director.name.clone());
        }
        if let Some(poster) = data.posters.map(|posters| posters.large) {
            let poster = self
                .client
                .wait()
                .await
                .get(poster)
                .send()
                .await?
                .bytes()
                .await?
                .to_vec();
            nfo.set_poster(poster);
        }
        if let Some(fanart) = data.background.map(|background| background.large) {
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

        Ok(())
    }

    async fn find_by_key(&self, key: &VideoType) -> Result<String> {
        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Response {
            pub component: Value,
            pub props: Props,
            pub url: Value,
            pub version: Value,
            #[serde(rename = "clearHistory")]
            pub clear_history: Value,
            #[serde(rename = "encryptHistory")]
            pub encrypt_history: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Props {
            pub errors: Value,
            pub jetstream: Value,
            pub auth: Value,
            #[serde(rename = "errorBags")]
            pub error_bags: Value,
            pub meta: Value,
            #[serde(rename = "verifiedAge")]
            pub verified_age: Value,
            pub hide_ads: Value,
            #[serde(rename = "currentRouteName")]
            pub current_route_name: Value,
            pub flash: Value,
            pub urls: Value,
            pub menu: Value,
            pub sfw: Value,
            pub dark: Value,
            pub scenes: Scenes,
            pub request: Value,
            pub sort: Value,
            pub genders: Value,
            pub operators: Value,
            pub hashes: Value,
            #[serde(rename = "siteOperators")]
            pub site_operators: Value,
            #[serde(rename = "queryOperations")]
            pub query_operations: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Scenes {
            pub data: Vec<Daum>,
            pub links: Value,
            pub meta: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Daum {
            pub background: Value,
            pub date: Value,
            pub default_background: Value,
            pub duration: Value,
            pub edit_link: Value,
            pub id: Value,
            pub is_collected: Value,
            pub is_hidden: Value,
            pub link: String,
            pub performers: Value,
            pub site: Value,
            pub slug: Value,
            pub title: String,
            #[serde(rename = "type")]
            pub type_field: Value,
        }

        let name = key.to_string();
        let url = format!("{}/jav", self.base_url);
        let text = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("q", &name)])
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);
        let data = html
            .select(&self.selectors.data)
            .next()
            .and_then(|app| app.attr("data-page"))
            .ok_or(anyhow!("data-page attribute not found"))?;
        let res = serde_json::from_str::<Response>(data).with_context(|| "parse data to json")?;

        res.props
            .scenes
            .data
            .into_iter()
            .find(|data| data.title.contains(&name))
            .map(|data| data.link)
            .ok_or(anyhow!("data not found"))
    }

    async fn get_uuid_by_link(&self, link: String) -> Result<String> {
        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Response {
            pub component: Value,
            pub props: Props,
            pub url: Value,
            pub version: Value,
            #[serde(rename = "clearHistory")]
            pub clear_history: Value,
            #[serde(rename = "encryptHistory")]
            pub encrypt_history: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Props {
            pub errors: Value,
            pub jetstream: Value,
            pub auth: Value,
            #[serde(rename = "errorBags")]
            pub error_bags: Value,
            pub meta: Value,
            #[serde(rename = "verifiedAge")]
            pub verified_age: Value,
            pub hide_ads: Value,
            #[serde(rename = "currentRouteName")]
            pub current_route_name: Value,
            pub flash: Value,
            pub urls: Value,
            pub menu: Value,
            pub sfw: Value,
            pub dark: Value,
            pub scene: Scene,
            #[serde(rename = "hashTypes")]
            pub hash_types: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Scene {
            pub background: Value,
            pub background_back: Value,
            pub date: Value,
            pub default_background: Value,
            pub description: Value,
            pub directors: Value,
            pub duration: Value,
            pub edit_link: Value,
            pub format: Value,
            pub hashes: Value,
            pub id: Value,
            pub is_collected: Value,
            pub is_hidden: Value,
            pub link: Value,
            pub links: Value,
            pub markers: Value,
            pub movies: Value,
            pub performers: Value,
            pub scenes: Value,
            pub site: Value,
            pub sku: Value,
            pub slug: Value,
            pub store: Value,
            pub tags: Value,
            pub title: Value,
            pub trailer: Value,
            #[serde(rename = "type")]
            pub type_field: Value,
            pub url: Value,
            pub uuid: String,
        }

        let text = self
            .client
            .wait()
            .await
            .get(link)
            .send()
            .await?
            .text()
            .await?;
        let html = Html::parse_document(&text);
        let data = html
            .select(&self.selectors.data)
            .next()
            .and_then(|app| app.attr("data-page"))
            .ok_or(anyhow!("data-page attribute not found"))?;
        let res = serde_json::from_str::<Response>(data).with_context(|| "parse data to json")?;

        Ok(res.props.scene.uuid)
    }
}
