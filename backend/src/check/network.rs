use super::Checker;
use crate::bar::Bar;
use async_trait::async_trait;
use console::style;
use reqwest::Client;
use std::sync::Arc;

pub struct Network {
    client: Arc<Client>,
}

impl Network {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Checker for Network {
    async fn check(&self) -> anyhow::Result<()> {
        let bar = Bar::new_check()?;
        bar.set_message("check network");

        let url = "https://www.javbus.com";
        let status = self.client.get(url).send().await?.status();
        if !status.is_success() {
            anyhow::bail!("ping url {url} failed");
        }

        bar.finish_and_clear();
        log::info!("network check passed");
        println!(
            "{:>10} ✔ network check passed",
            style("Check").green().bold()
        );

        Ok(())
    }
}
