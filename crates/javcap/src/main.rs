use anyhow::Result;

#[tokio::main]
async fn main() {
    println!("{}", ">".repeat(40));
    println!("当前版本: {}", app::VERSION);
    if let Err(e) = run().await {
        eprintln!("{}", e);
    }
    println!("{}", "<".repeat(40));
}

async fn run() -> Result<()> {
    let config = config::load().await?;

    Ok(())
}
