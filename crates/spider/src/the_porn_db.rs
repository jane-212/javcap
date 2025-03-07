use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::Value;
use video::VideoType;

use super::{Finder, which_country};

const HOST: &str = "https://theporndb.net";
const API_HOST: &str = "https://api.theporndb.net";

pub struct ThePornDB {
    base_url: String,
    api_url: String,
    client: Client,
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
            headers.insert(
                HeaderName::from_bytes(b"x-inertia")?,
                HeaderValue::from_static("true"),
            );
            headers.insert(
                HeaderName::from_bytes(b"x-inertia-version")?,
                HeaderValue::from_static("86b9c117e30461c03c71ed4e182e3a1a"),
            );

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

        let this = Self {
            base_url,
            api_url,
            client,
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
            pub id2: i64,
            pub title: String,
            #[serde(rename = "type")]
            pub type_field: Value,
            pub slug: Value,
            pub external_id: Value,
            pub description: Value,
            pub rating: i64,
            pub site_id: i64,
            pub date: String,
            pub url: Value,
            pub image: Value,
            pub back_image: Value,
            pub poster: Value,
            pub trailer: Value,
            pub duration: i64,
            pub format: Value,
            pub sku: Value,
            pub posters: Posters,
            pub background: Background,
            pub background_back: BackgroundBack,
            pub created: Value,
            pub last_updated: Value,
            pub performers: Vec<Performer>,
            pub site: Site,
            pub tags: Vec<Value>,
            pub hashes: Vec<Hash>,
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
        pub struct BackgroundBack {
            pub full: Value,
            pub large: Value,
            pub medium: Value,
            pub small: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Performer {
            pub id: Value,
            #[serde(rename = "_id")]
            pub id2: i64,
            pub slug: Value,
            pub site_id: i64,
            pub name: String,
            pub bio: Value,
            pub is_parent: bool,
            pub extra: Extra,
            pub image: Value,
            pub thumbnail: Value,
            pub face: Value,
            pub parent: Parent,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Extra {
            pub astrology: Value,
            pub birthday: Value,
            pub birthplace: Value,
            pub cupsize: Value,
            pub ethnicity: Value,
            pub eye_colour: Value,
            pub fakeboobs: bool,
            pub gender: Value,
            pub haircolor: Value,
            pub height: Value,
            pub measurements: Value,
            pub nationality: Value,
            pub piercings: Value,
            pub tattoos: Value,
            pub weight: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Parent {
            pub id: Value,
            #[serde(rename = "_id")]
            pub id2: i64,
            pub slug: Value,
            pub name: Value,
            pub disambiguation: Value,
            pub bio: Value,
            pub rating: i64,
            pub is_parent: bool,
            pub extras: Extras,
            pub image: Value,
            pub thumbnail: Value,
            pub face: Value,
            pub posters: Vec<Poster>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Extras {
            pub gender: Value,
            pub birthday: Value,
            pub birthday_timestamp: Value,
            pub deathday: Value,
            pub deathday_timestamp: Value,
            pub birthplace: Value,
            pub birthplace_code: Value,
            pub astrology: Value,
            pub ethnicity: Value,
            pub nationality: Value,
            pub hair_colour: Value,
            pub eye_colour: Value,
            pub weight: Value,
            pub height: Value,
            pub measurements: Value,
            pub cupsize: Value,
            pub tattoos: Value,
            pub piercings: Value,
            pub waist: Value,
            pub hips: Value,
            pub fake_boobs: bool,
            pub same_sex_only: bool,
            pub career_start_year: Value,
            pub career_end_year: Value,
            pub links: Vec<Value>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Poster {
            pub id: i64,
            pub url: Value,
            pub size: i64,
            pub order: i64,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Site {
            pub uuid: Value,
            pub id: i64,
            pub parent_id: i64,
            pub network_id: i64,
            pub name: String,
            pub short_name: Value,
            pub url: Value,
            pub description: Value,
            pub rating: i64,
            pub logo: Value,
            pub favicon: Value,
            pub poster: Value,
            pub network: Network,
            pub parent: Parent2,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Network {
            pub uuid: Value,
            pub id: i64,
            pub name: Value,
            pub short_name: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Parent2 {
            pub uuid: Value,
            pub id: i64,
            pub name: Value,
            pub short_name: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Hash {
            pub can_delete: bool,
            pub created_at: Value,
            pub duration: i64,
            pub hash: Value,
            pub id: i64,
            pub scene_id: i64,
            pub submissions: i64,
            #[serde(rename = "type")]
            pub type_field: Value,
            pub updated_at: Value,
            pub users: Vec<Value>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Director {
            pub id: i64,
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
        {
            let poster = data.posters.large;
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
        {
            let fanart = data.background.large;
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
            pub clear_history: bool,
            #[serde(rename = "encryptHistory")]
            pub encrypt_history: bool,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Props {
            pub errors: Errors,
            pub jetstream: Jetstream,
            pub auth: Auth,
            #[serde(rename = "errorBags")]
            pub error_bags: Vec<Value>,
            pub meta: Meta,
            #[serde(rename = "verifiedAge")]
            pub verified_age: bool,
            pub hide_ads: bool,
            #[serde(rename = "currentRouteName")]
            pub current_route_name: Value,
            pub flash: Flash,
            pub urls: Urls,
            pub menu: Vec<Menu>,
            pub sfw: bool,
            pub dark: bool,
            pub scenes: Scenes,
            pub request: Request,
            pub sort: Vec<Sort>,
            pub genders: Vec<Gender>,
            pub operators: Vec<Operator>,
            pub hashes: Vec<Hash>,
            #[serde(rename = "siteOperators")]
            pub site_operators: Vec<SiteOperator>,
            #[serde(rename = "queryOperations")]
            pub query_operations: Vec<QueryOperation>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Errors {}

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Jetstream {
            #[serde(rename = "canCreateTeams")]
            pub can_create_teams: bool,
            #[serde(rename = "canManageTwoFactorAuthentication")]
            pub can_manage_two_factor_authentication: bool,
            #[serde(rename = "canUpdatePassword")]
            pub can_update_password: bool,
            #[serde(rename = "canUpdateProfileInformation")]
            pub can_update_profile_information: bool,
            #[serde(rename = "hasEmailVerification")]
            pub has_email_verification: bool,
            pub flash: Vec<Value>,
            #[serde(rename = "hasAccountDeletionFeatures")]
            pub has_account_deletion_features: bool,
            #[serde(rename = "hasApiFeatures")]
            pub has_api_features: bool,
            #[serde(rename = "hasTeamFeatures")]
            pub has_team_features: bool,
            #[serde(rename = "hasTermsAndPrivacyPolicyFeature")]
            pub has_terms_and_privacy_policy_feature: bool,
            #[serde(rename = "managesProfilePhotos")]
            pub manages_profile_photos: bool,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Auth {
            pub id: i64,
            pub name: Value,
            pub email: Value,
            pub avatar: Value,
            pub two_factor_enabled: bool,
            pub has_admin_access: bool,
            pub email_verified_at: Value,
            pub is_collection_public: bool,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Meta {
            pub title: Value,
            pub description: Value,
            pub meta: Vec<Meum>,
            pub jsonld: Jsonld,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Meum {
            pub name: Option<String>,
            pub content: Value,
            pub property: Option<String>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Jsonld {
            #[serde(rename = "@context")]
            pub context: Value,
            #[serde(rename = "@type")]
            pub type_field: Value,
            pub name: Value,
            pub description: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Flash {
            pub error: Value,
            pub success: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Urls {
            pub main: Value,
            pub api: Value,
            pub admin: Value,
            pub current: Value,
            pub uri: Uri,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Uri {
            pub name: Value,
            pub uri: Value,
            pub path: Value,
            pub query: Query,
            pub params: Vec<Value>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Query {
            #[serde(rename = "orderBy")]
            pub order_by: Option<String>,
            pub page: Option<String>,
            pub q: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Menu {
            pub url: Value,
            pub title: Value,
            pub active: bool,
            pub attributes: Value,
            pub children: Vec<Value>,
            pub depth: i64,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Scenes {
            pub data: Vec<Daum>,
            pub links: Vec<Link>,
            pub meta: Meta2,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Daum {
            pub background: Background,
            pub date: Value,
            pub default_background: Value,
            pub duration: i64,
            pub edit_link: Value,
            pub id: i64,
            pub is_collected: bool,
            pub is_hidden: bool,
            pub link: String,
            pub performers: Vec<Performer>,
            pub site: Site,
            pub slug: Value,
            pub title: String,
            #[serde(rename = "type")]
            pub type_field: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Background {
            pub large: Value,
            pub medium: Value,
            pub poster: Value,
            pub small: Value,
            pub thumb: Value,
            #[serde(rename = "thumbHash")]
            pub thumb_hash: Value,
            pub url: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Performer {
            pub disambiguation: Value,
            pub full_name: Value,
            pub gender: Value,
            pub id: i64,
            pub is_parent: Value,
            pub link: Value,
            pub name: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Site {
            pub description: Value,
            pub edit_link: Value,
            pub favicon: Value,
            pub id: i64,
            pub link: Value,
            pub logo: Option<Value>,
            pub name: Value,
            pub short_name: Value,
            pub url: Value,
            pub uuid: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Link {
            pub url: Option<Value>,
            pub label: Value,
            pub active: bool,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Meta2 {
            pub current_page: i64,
            pub first_page_url: Value,
            pub from: i64,
            pub last_page: i64,
            pub last_page_url: Value,
            pub next_page_url: Value,
            pub path: Value,
            pub per_page: i64,
            pub prev_page_url: Value,
            pub to: i64,
            pub total: i64,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Request {
            pub category_id: Value,
            pub date: Value,
            pub date_operation: Value,
            pub director_and: Value,
            pub director_id: Value,
            pub directors: Vec<Value>,
            pub duration: Value,
            pub duration_operation: Value,
            pub external_id: Value,
            pub hash: Value,
            #[serde(rename = "hashType")]
            pub hash_type: Value,
            pub is_collected: Value,
            pub is_favourite: Value,
            pub is_hidden: Value,
            pub no_performer_genders: Value,
            pub no_performers: Value,
            #[serde(rename = "orderBy")]
            pub order_by: Value,
            pub page: i64,
            pub parse: Value,
            pub per_page: Value,
            pub performer_and: Value,
            pub performer_gender_and: Value,
            pub performer_gender_only: Value,
            pub performer_genders: Vec<Value>,
            pub performer_id: Value,
            pub performers: Vec<Value>,
            pub q: Value,
            pub query_operation: Value,
            pub site: Value,
            pub site_and: Value,
            pub site_id: Value,
            pub site_operation: Value,
            pub sites: Vec<Value>,
            pub sku: Value,
            pub tag_and: Value,
            pub tags: Vec<Value>,
            pub title: Value,
            #[serde(rename = "type")]
            pub type_field: Value,
            pub url: Value,
            pub year: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Sort {
            pub value: Value,
            pub label: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Gender {
            pub value: Value,
            pub label: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Operator {
            pub value: Value,
            pub label: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Hash {
            pub value: Value,
            pub label: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct SiteOperator {
            pub value: Value,
            pub label: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct QueryOperation {
            pub value: Value,
            pub label: Value,
        }

        let name = key.to_string();
        let url = format!("{}/jav", self.base_url);
        let res = self
            .client
            .wait()
            .await
            .get(url)
            .query(&[("q", &name)])
            .send()
            .await?
            .json::<Response>()
            .await?;

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
            pub clear_history: bool,
            #[serde(rename = "encryptHistory")]
            pub encrypt_history: bool,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Props {
            pub errors: Errors,
            pub jetstream: Jetstream,
            pub auth: Auth,
            #[serde(rename = "errorBags")]
            pub error_bags: Vec<Value>,
            pub meta: Meta,
            #[serde(rename = "verifiedAge")]
            pub verified_age: bool,
            pub hide_ads: bool,
            #[serde(rename = "currentRouteName")]
            pub current_route_name: Value,
            pub flash: Flash,
            pub urls: Urls,
            pub menu: Vec<Menu>,
            pub sfw: bool,
            pub dark: bool,
            pub scene: Scene,
            #[serde(rename = "hashTypes")]
            pub hash_types: Vec<HashType>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Errors {}

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Jetstream {
            #[serde(rename = "canCreateTeams")]
            pub can_create_teams: bool,
            #[serde(rename = "canManageTwoFactorAuthentication")]
            pub can_manage_two_factor_authentication: bool,
            #[serde(rename = "canUpdatePassword")]
            pub can_update_password: bool,
            #[serde(rename = "canUpdateProfileInformation")]
            pub can_update_profile_information: bool,
            #[serde(rename = "hasEmailVerification")]
            pub has_email_verification: bool,
            pub flash: Vec<Value>,
            #[serde(rename = "hasAccountDeletionFeatures")]
            pub has_account_deletion_features: bool,
            #[serde(rename = "hasApiFeatures")]
            pub has_api_features: bool,
            #[serde(rename = "hasTeamFeatures")]
            pub has_team_features: bool,
            #[serde(rename = "hasTermsAndPrivacyPolicyFeature")]
            pub has_terms_and_privacy_policy_feature: bool,
            #[serde(rename = "managesProfilePhotos")]
            pub manages_profile_photos: bool,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Auth {
            pub id: i64,
            pub name: Value,
            pub email: Value,
            pub avatar: Value,
            pub two_factor_enabled: bool,
            pub has_admin_access: bool,
            pub email_verified_at: Value,
            pub is_collection_public: bool,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Meta {
            pub title: Value,
            pub description: Value,
            pub meta: Vec<Meum>,
            pub jsonld: Jsonld,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Meum {
            pub name: Option<Value>,
            pub content: Value,
            pub property: Option<Value>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Jsonld {
            #[serde(rename = "@context")]
            pub context: Value,
            #[serde(rename = "@type")]
            pub type_field: Value,
            pub name: Value,
            pub url: Value,
            pub image: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Flash {
            pub error: Value,
            pub success: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Urls {
            pub main: Value,
            pub api: Value,
            pub admin: Value,
            pub current: Value,
            pub uri: Uri,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Uri {
            pub name: Value,
            pub uri: Value,
            pub path: Value,
            pub query: Vec<Value>,
            pub params: Params,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Params {
            pub slug: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Menu {
            pub url: Value,
            pub title: Value,
            pub active: bool,
            pub attributes: Value,
            pub children: Vec<Value>,
            pub depth: i64,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Scene {
            pub background: Background,
            pub background_back: Value,
            pub date: Value,
            pub default_background: Value,
            pub description: Value,
            pub directors: Vec<Director>,
            pub duration: i64,
            pub edit_link: Value,
            pub format: Value,
            pub hashes: Vec<Hash>,
            pub id: i64,
            pub is_collected: bool,
            pub is_hidden: bool,
            pub link: Value,
            pub links: Vec<Value>,
            pub markers: Vec<Value>,
            pub movies: Vec<Value>,
            pub performers: Vec<Performer>,
            pub scenes: Vec<Value>,
            pub site: Site,
            pub sku: Value,
            pub slug: Value,
            pub store: Value,
            pub tags: Vec<Value>,
            pub title: Value,
            pub trailer: Value,
            #[serde(rename = "type")]
            pub type_field: Value,
            pub url: Value,
            pub uuid: String,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Background {
            pub large: Value,
            pub medium: Value,
            pub poster: Value,
            pub small: Value,
            pub thumb: Value,
            #[serde(rename = "thumbHash")]
            pub thumb_hash: Value,
            pub url: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Director {
            pub id: i64,
            pub name: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Hash {
            pub can_delete: bool,
            pub created_at: Value,
            pub duration: i64,
            pub hash: Value,
            pub id: i64,
            pub scene_id: i64,
            pub submissions: i64,
            #[serde(rename = "type")]
            pub type_field: Value,
            pub updated_at: Value,
            pub users: Vec<Value>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Performer {
            pub age: Value,
            pub birthday: Value,
            pub birthplace_code: Value,
            pub deathday: Value,
            pub default_image: Value,
            pub disambiguation: Value,
            pub edit_link: Value,
            pub full_name: Value,
            pub gender: Value,
            pub id: i64,
            pub image: Option<Image>,
            pub is_hidden: bool,
            pub is_performer: bool,
            pub link: Value,
            pub name: Value,
            pub parent: Parent,
            pub site: Value,
            pub slug: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Image {
            pub large: Value,
            pub medium: Value,
            pub poster: Value,
            pub small: Value,
            pub thumb: Value,
            #[serde(rename = "thumbHash")]
            pub thumb_hash: Value,
            pub url: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Parent {
            pub age: Value,
            pub birthday: Value,
            pub birthplace_code: Value,
            pub deathday: Value,
            pub default_image: Value,
            pub disambiguation: Value,
            pub edit_link: Value,
            pub full_name: Value,
            pub gender: Value,
            pub id: i64,
            pub image: Image2,
            pub is_hidden: bool,
            pub is_performer: bool,
            pub link: Value,
            pub name: Value,
            pub parent: Value,
            pub site: Value,
            pub slug: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Image2 {
            pub large: Value,
            pub medium: Value,
            pub poster: Value,
            pub small: Value,
            pub thumb: Value,
            #[serde(rename = "thumbHash")]
            pub thumb_hash: Value,
            pub url: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct Site {
            pub description: Value,
            pub edit_link: Value,
            pub favicon: Value,
            pub id: i64,
            pub link: Value,
            pub logo: Value,
            pub name: Value,
            pub short_name: Value,
            pub url: Value,
            pub uuid: Value,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        pub struct HashType {
            pub value: Value,
            pub label: Value,
        }

        let res = self
            .client
            .wait()
            .await
            .get(link)
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(res.props.scene.uuid)
    }
}
