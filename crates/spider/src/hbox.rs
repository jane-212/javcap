use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use serde::Deserialize;
use video::VideoType;

use super::{Finder, which_country};

const HOST: &str = "https://hbox.jp";

pub struct Hbox {
    base_url: String,
    client: Client,
}

#[bon]
impl Hbox {
    #[builder]
    pub fn new(base_url: Option<String>, timeout: Duration, proxy: Option<String>) -> Result<Hbox> {
        let client = Client::builder()
            .timeout(timeout)
            .interval(1)
            .maybe_proxy(proxy)
            .build()
            .with_context(|| "build http client")?;
        let base_url = match base_url {
            Some(url) => url,
            None => String::from(HOST),
        };

        let hbox = Hbox { base_url, client };
        Ok(hbox)
    }

    async fn find_name(&self, name: &str) -> Result<Content> {
        let url = format!("{}/home_api/search_result", self.base_url);
        let mut payload = self
            .client
            .wait()
            .await
            .get(&url)
            .query(&[("q_array[]", name)])
            .send()
            .await?
            .json::<Payload>()
            .await?;
        if payload.count == 0 {
            bail!("payload count is zero");
        }

        payload
            .contents
            .pop()
            .ok_or(anyhow!("empty payload content"))
    }
}

impl Display for Hbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hbox")
    }
}

#[async_trait]
impl Finder for Hbox {
    fn support(&self, key: &VideoType) -> bool {
        match key {
            VideoType::Jav(_, _) => !matches!(which_country(key), Country::China),
            VideoType::Fc2(_) => false,
            VideoType::Other(_) => false,
        };

        // TODO: hbox可用时启用
        false
    }

    async fn find(&self, key: &VideoType) -> Result<Nfo> {
        let mut nfo = Nfo::builder()
            .id(key)
            .country(Country::Japan)
            .mpaa(Mpaa::NC17)
            .build();

        let content = self
            .find_name(&key.to_string())
            .await
            .with_context(|| "find name")?;
        nfo.set_title(content.title);
        nfo.set_plot(content.description);
        nfo.set_premiered(content.release_date);
        nfo.set_studio(content.label_name);
        nfo.set_director(content.director_names);
        content.casts.into_iter().for_each(|actor| {
            nfo.actors_mut().insert(actor.cast_name);
        });
        content.tags.into_iter().for_each(|tag| {
            nfo.genres_mut().insert(tag.name);
        });
        let poster = format!(
            "{}{}/{}",
            self.base_url, content.back_cover_url_root, content.back_cover_file,
        );
        let poster = self
            .client
            .wait()
            .await
            .get(&poster)
            .send()
            .await?
            .bytes()
            .await?;
        nfo.set_poster(poster.to_vec());

        info!("{nfo:?}");
        Ok(nfo)
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Payload {
    q_array: Vec<String>,
    sort_key: Option<String>,
    sdc_sort: Option<String>,
    pickup: Option<String>,
    category_id: String,
    subcategory_id: String,
    category_name: String,
    category_code: String,
    subcategory_name: String,
    subcategory_code: String,
    #[serde(rename = "lastOpeningDate")]
    last_opening_date: String,
    banner_img_width: i32,
    banner_img_height: i32,
    title: String,
    #[serde(rename = "saleInfo")]
    sale_info: Option<String>,
    features: Option<String>,
    #[serde(rename = "saleEvents")]
    sale_events: Option<String>,
    #[serde(rename = "exclude_AIG")]
    exclude_aig: String,
    #[serde(rename = "exclude_AIP")]
    exclude_aip: String,
    contents: Vec<Content>,
    count: i32,
    page: i32,
    count_par_page: i32,
    maxpage: i32,
    query: Query,
    refine_list: RefineList,
    sale: String,
    coin: String,
    #[serde(rename = "openRefine")]
    open_refine: String,
    tag_name: Option<String>,
    #[serde(rename = "auther_names")]
    author_names: String,
    cast_names: String,
    director_name: String,
    label_name: String,
    publisher_name: Option<String>,
    series_name: String,
    devices: Option<String>,
    course_names: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Content {
    content_id: String,
    title: String,
    description: String,
    opening_status: String,
    opening_date: String,
    release_date: String,
    brand_new_date: String,
    brand_new_status: String,
    price: i32,
    rental_price: i32,
    before_view: String,
    before_sale: String,
    rental_flg: String,
    img_src: String,
    category_id: String,
    category_code: String,
    category_name: String,
    subcategory_id: String,
    subcategory_code: String,
    subcategory_name: String,
    content_type: String,
    android_flg: String,
    ios_flg: String,
    pc_flg: String,
    vr_flg: String,
    vr_type: String,
    vr_mode: String,
    maker_id: String,
    maker_name: String,
    label_id: String,
    label_name: String,
    series_id: String,
    series_name: String,
    galleries: Vec<Gallery>,
    in_cart: bool,
    is_paid: bool,
    is_bookmarked: bool,
    purchase_price: i32,
    directors: Vec<Director>,
    casts: Vec<Cast>,
    medal_magnification: Option<String>,
    hd_info: HdInfo,
    hd_content_price: i32,
    content_price: i32,
    comic_sample_url: String,
    cover_url_root: String,
    cover_file: String,
    back_cover_url_root: String,
    back_cover_file: String,
    ios_sample_url: String,
    android_sample_url: String,
    promotions: Vec<String>,
    director_names: String,
    cast_names: String,
    tags: Vec<Tag>,
    review_score: i32,
    review_count: i32,
    is_only_sd_paid: bool,
    screen_time: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Gallery {
    id: String,
    content_id: String,
    client_type: String,
    image_no: String,
    image_url_root: String,
    image_dir: String,
    image_file: String,
    sample_flg: String,
    del_flg: String,
    created: String,
    modified: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Director {
    id: String,
    director_name: String,
    director_kana: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Cast {
    id: String,
    cast_name: String,
    cast_kana: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HdInfo {
    content_id: String,
    hd_brand_new_price: i32,
    hd_recent_price: i32,
    hd_price: i32,
    hd_xiaomaisige: Option<String>,
    hd_reserve_price: i32,
    hd_rental_price: i32,
    hd_newcomer_price: i32,
    has_hd: bool,
    is_hd_paid: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Tag {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Query {
    q_array: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RefineList {}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn finder() -> Result<Hbox> {
        Hbox::builder().timeout(Duration::from_secs(10)).build()
    }

    #[test]
    #[ignore = "hbox暂时不可用"]
    fn test_support() -> Result<()> {
        let finder = finder()?;
        let videos = [
            (VideoType::Jav("STARS".to_string(), "804".to_string()), true),
            (VideoType::Fc2("3061625".to_string()), false),
        ];
        for (video, supported) in videos {
            assert_eq!(finder.support(&video), supported);
        }

        Ok(())
    }

    #[tokio::test]
    #[ignore = "hbox暂时不可用"]
    async fn test_find() -> Result<()> {
        let finder = finder()?;
        let cases = [(VideoType::Jav("STARS".to_string(), "804".to_string()), {
            let mut nfo = Nfo::builder()
                .id("STARS-804")
                .country(Country::Japan)
                .mpaa(Mpaa::NC17)
                .build();
            nfo.set_title("本能で絡み合う極上のランジェリー＆オイリー4本番 神木麗".to_string())
                .set_plot("【神木麗1周年企画】Gカップボディと、抜群のプロポーションをより際立たせる高級ランジェリーとオイル。ホテルへ入るなり求め合う濃厚なSEX。美しい四肢を夜景バックに縛り付け、執拗に責め立てられ絶頂し続け自ら挿入を懇願。喘ぐ声も抑えることなく、密室に響き渡る。神木麗お初の4本番。最後は美しき涙も…".to_string())
                .set_studio("SODクリエイト".to_string())
                .set_director("チク兄".to_string())
                .set_premiered("2023-04-06".to_string());
            let genres = ["巨乳", "単体作品"];
            nfo.actors_mut().insert("神木麗".to_string());

            for genre in genres {
                nfo.genres_mut().insert(genre.to_string());
            }
            nfo
        })];
        for (video, expected) in cases {
            let actual = finder.find(&video).await?;
            assert!(actual.fanart().is_empty());
            assert!(!actual.poster().is_empty());
            assert!(actual.subtitle().is_empty());
            assert_eq!(actual, expected);
        }

        Ok(())
    }
}
