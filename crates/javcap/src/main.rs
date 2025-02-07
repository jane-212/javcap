use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use colored::Colorize;
use config::Config;
use javcap::App;
use tokio::sync::{Notify, RwLock};
use tokio::time;
use validator::Validate;

#[tokio::main]
async fn main() {
    let quit = Arc::new(RwLock::new(false));
    let notify = Arc::new(Notify::new());
    tokio::spawn({
        let quit = quit.clone();
        let notify = notify.clone();
        async move {
            start_progress_bar(quit, notify).await;
        }
    });
    println!("\r{}", ">".repeat(app::LINE_LENGTH).yellow());
    if let Err(e) = run().await {
        println!("\r{:#^width$}", " Error ".red(), width = app::LINE_LENGTH);
        eprintln!("\r{}", e);
    }
    {
        let mut quit = quit.write().await;
        *quit = true;
    }
    notify.notified().await;
    println!("\r{}", "<".repeat(app::LINE_LENGTH).yellow());
}

async fn start_progress_bar(quit: Arc<RwLock<bool>>, notify: Arc<Notify>) {
    let interval = Duration::from_millis(80);
    let mut idx = 0;
    let bar = ["⣼", "⣹", "⢻", "⠿", "⡟", "⣏", "⣧", "⣶"];

    loop {
        print!("\r{}", bar[idx]);
        io::stdout().flush().ok();
        idx += 1;
        idx %= bar.len();
        time::sleep(interval).await;
        if *quit.read().await {
            break;
        }
    }
    notify.notify_one();
}

async fn run() -> Result<()> {
    println!("\r当前版本: {}({})", app::VERSION, app::HASH);

    let config = Config::load().await?;
    config.validate()?;

    let app = App::new(config)?;

    app.run().await
}
