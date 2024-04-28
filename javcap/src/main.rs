use app::App;
use console::style;
use std::io::{self, Read, Write};
use tracing::{error, info};

mod app;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(should_quit) => {
            finish_and_quit(should_quit);
        }
        Err(err) => {
            error!("{err}");
            println!("{:>10} {}", style("Error").red().bold(), err);
            finish_and_quit(false);
        }
    }
}

fn finish_and_quit(should_quit: bool) {
    info!("{:-^30}", " Finish ");
    if !should_quit {
        wait_for_quit();
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

async fn run() -> anyhow::Result<bool> {
    let mut app = App::new().await?;

    app.run().await
}
