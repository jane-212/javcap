use std::time::Duration;

use console::style;
use error::Result;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use tokio::time::Instant;
use tracing::{info, warn};

pub struct Bar {
    timer: Instant,
    multi: MultiProgress,
    info: ProgressBar,
    process: ProgressBar,
}

impl Bar {
    pub fn new(len: u64) -> Result<Bar> {
        let multi = MultiProgress::new();
        let info = multi.add(ProgressBar::new_spinner());
        info.enable_steady_tick(Duration::from_millis(100));
        info.set_style(
            ProgressStyle::with_template("{prefix:>10.cyan.bold} {spinner} {msg}")?
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        info.set_prefix("Handle");
        let process = multi.add(ProgressBar::new(len));
        process.set_style(
            ProgressStyle::with_template("{prefix:>10.cyan.bold} |{wide_bar}| {pos}/{len} ")?
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        process.set_prefix("Process");

        Ok(Bar {
            timer: Instant::now(),
            multi,
            info,
            process,
        })
    }

    pub fn message(&self, msg: &str) {
        info!("{msg}");
        self.info.set_message(msg.to_string());
    }

    pub fn info(&self, msg: &str) {
        info!("{msg}");
        self.process.inc(1);
        self.process
            .println(format!("{:>10} {}", style("Handle").green().bold(), msg));
    }

    pub fn warn(&self, msg: &str) {
        warn!("{msg}");
        self.process.inc(1);
        self.process
            .println(format!("{:>10} {}", style("Handle").red().bold(), msg));
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        self.multi.clear().ok();
        println!(
            "{:>10} took {}",
            style("Finish").green().bold(),
            HumanDuration(self.timer.elapsed())
        );
        info!("took {}", HumanDuration(self.timer.elapsed()));
    }
}
