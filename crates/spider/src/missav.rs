use std::fmt::{self, Display};
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use bon::bon;
use http_client::Client;
use log::info;
use nfo::{Country, Mpaa, Nfo};
use video::VideoType;

use super::Finder;

const HOST: &str = "https://fourhoi.com";

pub struct Missav {
    base_url: String,
    client: Client,
}

#[bon]
impl Missav {
    #[builder]
    pub fn new(
        base_url: Option<String>,
        timeout: Duration,
        proxy: Option<String>,
    ) -> Result<Missav> {
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

        let missav = Missav { base_url, client };
        Ok(missav)
    }

    async fn get_fanart(&self, key: &VideoType) -> Result<Vec<u8>> {
        let url = format!(
            "{}/{}/cover-n.jpg",
            self.base_url,
            key.to_string().to_lowercase()
        );
        let img = self
            .client
            .wait()
            .await
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec();

        Ok(img)
    }
}

impl Display for Missav {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missav")
    }
}

#[async_trait]
impl Finder for Missav {
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

        let fanart = self.get_fanart(key).await.with_context(|| "get fanart")?;
        nfo.set_fanart(fanart);

        info!("{nfo:?}");
        Ok(nfo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn finder() -> Result<Missav> {
        Missav::builder().timeout(Duration::from_secs(5)).build()
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
            (VideoType::Jav("IPX".to_string(), "443".to_string()), {
                Nfo::builder()
                    .id("IPX-443")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build()
            }),
            (VideoType::Fc2("3061625".to_string()), {
                Nfo::builder()
                    .id("FC2-PPV-3061625")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build()
            }),
            (VideoType::Fc2("1292936".to_string()), {
                Nfo::builder()
                    .id("FC2-PPV-1292936")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build()
            }),
            (VideoType::Jav("ROYD".to_string(), "108".to_string()), {
                Nfo::builder()
                    .id("ROYD-108")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build()
            }),
            (VideoType::Jav("STARS".to_string(), "804".to_string()), {
                Nfo::builder()
                    .id("STARS-804")
                    .country(Country::Japan)
                    .mpaa(Mpaa::NC17)
                    .build()
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
