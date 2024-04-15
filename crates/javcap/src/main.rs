use std::io::{self, Read, Write};

use app::App;
use console::style;
use error::Result;
use tracing::{error, info};

mod app;
mod bar;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(should_quit) => {
            info!("{:-^30}", " Finish ");
            if !should_quit {
                wait_for_quit();
            }
        }
        Err(err) => {
            error!("{err}");
            info!("{:-^30}", " Finish ");
            println!("{:>10} {}", style("Error").red().bold(), err);
            wait_for_quit();
        }
    }
}

fn wait_for_quit() {
    print!(
        "{:>10} Press enter to continue...",
        style("Pause").green().bold()
    );
    io::stdout().flush().ok();
    io::stdin().read_exact(&mut [0u8]).ok();
}

async fn run() -> Result<bool> {
    let app = App::new().await?;
    app.run().await
}
