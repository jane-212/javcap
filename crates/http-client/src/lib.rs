use std::time::Duration;

use anyhow::Result;
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
        let limiter = Ratelimiter::builder(amount.unwrap_or(1), Duration::from_secs(interval))
            .initial_available(1)
            .build()?;
        let mut client_builder = HttpClient::builder()
            .timeout(timeout)
            .user_agent(app::USER_AGENT);
        if let Some(url) = proxy {
            let proxy = Proxy::all(url)?;
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder.build()?;
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
