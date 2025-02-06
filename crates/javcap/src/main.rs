use anyhow::Result;
use colored::Colorize;
use config::Config;
use javcap::App;
use validator::Validate;

#[tokio::main]
async fn main() {
    println!("{}", ">".repeat(app::LINE_LENGTH).yellow());
    if let Err(e) = run().await {
        println!("{:#^width$}", " Error ".red(), width = app::LINE_LENGTH);
        eprintln!("{}", e);
    }
    println!("{}", "<".repeat(app::LINE_LENGTH).yellow());
}

async fn run() -> Result<()> {
    println!("当前版本: {}({})", app::VERSION, app::HASH);

    let config = Config::load().await?;
    config.validate()?;

    let app = App::new(config)?;

    app.run().await
}
