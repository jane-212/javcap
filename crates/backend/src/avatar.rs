use std::{collections::HashMap, sync::Arc};

use base64::{prelude::BASE64_STANDARD, Engine};
use error::{Error, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

use crate::bar::Bar;

pub struct Avatar {
    client: Arc<Client>,
    host: String,
    api_key: String,
}

impl Avatar {
    const HOST: &'static str = "https://raw.githubusercontent.com/gfriends/gfriends/master";

    pub fn new(client: Arc<Client>, host: String, api_key: String) -> Avatar {
        Avatar {
            client,
            host,
            api_key,
        }
    }

    pub async fn refresh(&self) -> Result<()> {
        let actors = self.get_actors().await.map_err(|_| Error::Emby)?;
        info!("total {} actors", actors.len());
        let mut bar = Bar::new(actors.len() as u64)?;
        bar.println("AVATAR");
        bar.message("load file tree");
        let actor_map = self.load_file_tree().await.map_err(|_| Error::Avatar)?;
        info!("actor map loaded");
        for actor in actors {
            if let Err(err) = self.handle(actor, &actor_map, &mut bar).await {
                bar.warn(&format!("{}", err));
            }
        }

        Ok(())
    }

    async fn handle(
        &self,
        actor: (String, String),
        actor_map: &HashMap<String, HashMap<String, String>>,
        bar: &mut Bar,
    ) -> Result<()> {
        let (id, name) = actor;
        let file_name = format!("{}.jpg", name);
        if let Some(company) = actor_map.iter().find(|map| map.1.get(&file_name).is_some()) {
            if let Some(file_name) = company.1.get(&file_name) {
                let url = format!("{}/Content/{}/{}", Avatar::HOST, company.0, file_name);
                let img = self.load_img(&url).await?;
                self.save_img(&id, img).await?;
                bar.info(&format!("{name}({id})"));
                return Ok(());
            }
        }
        bar.warn(&format!("avatar not found, {name}({id})"));

        Ok(())
    }

    async fn save_img(&self, id: &str, img: String) -> Result<()> {
        let url = format!(
            "{}/Items/{}/Images/Primary?api_key={}",
            self.host, id, self.api_key
        );
        if !self
            .client
            .post(url)
            .header("Content-Type", "image/jpeg")
            .body(img)
            .send()
            .await?
            .status()
            .is_success()
        {
            return Err(Error::Emby);
        }

        Ok(())
    }

    async fn load_img(&self, url: &str) -> Result<String> {
        let bytes = self.client.get(url).send().await?.bytes().await?;
        let img = BASE64_STANDARD.encode(bytes);

        Ok(img)
    }

    async fn get_actors(&self) -> Result<Vec<(String, String)>> {
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "TotalRecordCount")]
            total_record_count: u32,
            #[serde(rename = "Items")]
            items: Vec<Item>,
        }
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct Item {
            #[serde(rename = "Name")]
            name: String,
            #[serde(rename = "ServerId")]
            server_id: String,
            #[serde(rename = "Id")]
            id: String,
            #[serde(rename = "Type")]
            t: String,
            #[serde(rename = "ImageTags")]
            image_tags: ImageTags,
            #[serde(rename = "BackdropImageTags")]
            backdrop_image_tags: Vec<ImageTags>,
        }
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct ImageTags {
            #[serde(rename = "Primary")]
            primary: Option<String>,
        }
        let url = format!("{}/Persons?api_key={}", self.host, self.api_key);
        let res = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(res
            .items
            .into_iter()
            .map(|item| (item.id, item.name))
            .collect())
    }

    async fn load_file_tree(&self) -> Result<HashMap<String, HashMap<String, String>>> {
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "Information")]
            information: Information,
            #[serde(rename = "Content")]
            content: HashMap<String, HashMap<String, String>>,
        }
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct Information {
            #[serde(rename = "TotalNum")]
            total_num: u32,
            #[serde(rename = "TotalSize")]
            total_size: u64,
            #[serde(rename = "Timestamp")]
            timestamp: f64,
        }
        let url = format!("{}/Filetree.json", Avatar::HOST);
        let res = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(res.content)
    }
}
