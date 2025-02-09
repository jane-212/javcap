use std::time::Duration;

use anyhow::{Context, Result};
use bon::bon;
use ratelimit::Ratelimiter;
use reqwest::Client as HttpClient;
use reqwest::Proxy;
use tokio::time;

pub struct Client {
    client: HttpClient,
    limiter: Ratelimiter,
}

#[bon]
impl Client {
    #[builder]
    pub fn new(
        timeout: Duration,
        proxy: Option<String>,
        amount: Option<u64>,
        interval: u64,
    ) -> Result<Client> {
        let amount = amount.unwrap_or(1);
        let limiter = Ratelimiter::builder(amount, Duration::from_secs(interval))
            .max_tokens(amount)
            .initial_available(amount)
            .build()
            .with_context(|| "build limiter")?;
        let mut client_builder = HttpClient::builder()
            .timeout(timeout)
            .user_agent(app::USER_AGENT);
        if let Some(url) = proxy {
            let proxy = Proxy::all(&url).with_context(|| format!("set proxy to {url}"))?;
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder
            .build()
            .with_context(|| "build reqwest client")?;
        let client = Client { client, limiter };

        Ok(client)
    }

    pub async fn wait(&self) -> &HttpClient {
        self.wait_limiter().await;

        &self.client
    }

    async fn wait_limiter(&self) {
        loop {
            match self.limiter.try_wait() {
                Ok(_) => break,
                Err(sleep) => time::sleep(sleep).await,
            }
        }
    }
}
