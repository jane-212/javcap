use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Url {
    #[validate(url(message = "should be a url"))]
    pub airav: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub avsox: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub cable: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub fc2ppv_db: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub hbox: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub jav321: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub javdb: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub missav: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub porny: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub subtitle_cat: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub the_porn_db: Option<String>,

    #[validate(url(message = "should be a url"))]
    pub the_porn_db_api: Option<String>,
}
