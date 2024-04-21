use std::{env, path::Path, sync::Arc, time::Duration};

use backend::check::network::Network;
use backend::check::Checker;
use backend::task::avatar::Avatar;
use backend::task::video::Video;
use backend::task::Task;
use config::Config;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Proxy,
};
use time::{macros::format_description, UtcOffset};
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::fmt::time::OffsetTime;

pub struct App {
    should_quit: bool,
    tasks: Vec<Box<dyn Task>>,
    checkers: Vec<Box<dyn Checker>>,
}

impl App {
    const CONFIG_NAME: &'static str = "config.toml";
    const LOG_NAME: &'static str = "logs";

    pub async fn new() -> anyhow::Result<Self> {
        let pwd = env::current_dir()?;
        Self::init_tracing(&pwd);
        info!(
            "{:-^30}",
            format!(
                " {} - {} ",
                env!("CARGO_PKG_NAME").to_uppercase(),
                env!("VERSION")
            )
        );
        let config = Config::load(&pwd.join(Self::CONFIG_NAME)).await?;
        info!(
            "config loaded from {}",
            pwd.join(Self::CONFIG_NAME).display()
        );
        let client = Self::default_client(&config)?;
        let video = Video::new(client.clone(), &config, &pwd)?;
        let avatar = Avatar::new(client.clone(), &config);
        let tasks: Vec<Box<dyn Task>> = vec![Box::new(video), Box::new(avatar)];
        let network_checker = Network::new(client);
        let checkers: Vec<Box<dyn Checker>> = vec![Box::new(network_checker)];

        Ok(Self {
            should_quit: config.app.quit_on_finish,
            tasks,
            checkers,
        })
    }

    fn default_client(config: &Config) -> anyhow::Result<Arc<Client>> {
        let headers = {
            let mut headers = HeaderMap::new();
            headers.insert(header::USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4.1 Safari/605.1.15"));
            headers.insert(
                header::ACCEPT_ENCODING,
                HeaderValue::from_static("gzip, deflate, br"),
            );
            headers.insert(
                header::ACCEPT,
                HeaderValue::from_static(
                    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                ),
            );
            headers.insert(
                header::ACCEPT_LANGUAGE,
                HeaderValue::from_static("zh-CN,zh-Hans;q=0.9"),
            );
            headers
        };
        let proxy = &config.network.proxy;
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(config.network.timeout))
            .proxy(
                Proxy::https(proxy)
                    .map_err(|_| anyhow::anyhow!("proxy {proxy} is not validate"))?,
            )
            .build()?;
        let client = Arc::new(client);

        Ok(client)
    }

    async fn check(&self) -> anyhow::Result<()> {
        for checker in self.checkers.iter() {
            checker.check().await?;
        }

        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<bool> {
        self.check().await?;
        for task in self.tasks.iter_mut() {
            task.run().await?;
        }

        Ok(self.should_quit)
    }

    fn init_tracing(path: &Path) {
        let daily = rolling::daily(path.join(Self::LOG_NAME), "log");
        let timer = OffsetTime::new(
            UtcOffset::from_hms(8, 0, 0).expect("set timezone error"),
            format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        );
        tracing_subscriber::fmt()
            .with_writer(daily)
            .with_max_level(Level::INFO)
            .with_ansi(false)
            .with_target(false)
            .with_timer(timer)
            .init();
    }
}
