use anyhow::Result;
use config::Config;
use javcap::App;
use validator::Validate;

#[tokio::main]
async fn main() {
    println!("{}", ">".repeat(app::LINE_LENGTH));
    if let Err(e) = run().await {
        println!("{:#^width$}", " Error ", width = app::LINE_LENGTH);
        eprintln!("{}", e);
    }
    println!("{}", "<".repeat(app::LINE_LENGTH));
}

async fn run() -> Result<()> {
    println!("当前版本: {}({})", app::VERSION, app::HASH);

    let config = Config::load().await?;
    config.validate()?;

    let app = App::new(config)?;

    app.run().await
}
