use std::time::Duration;

use console::style;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use tokio::time::Instant;
use tracing::{info, warn};

pub struct Bar {
    timer: Instant,
    multi: MultiProgress,
    info: ProgressBar,
    process: ProgressBar,
    success: u32,
    failed: u32,
    total: u32,
}

impl Bar {
    pub fn new(len: u64) -> anyhow::Result<Bar> {
        let multi = MultiProgress::new();
        let info = multi.add(ProgressBar::new(len));
        info.enable_steady_tick(Duration::from_millis(100));
        info.set_style(
            ProgressStyle::with_template("{prefix:>10.blue.bold} {spinner} [{pos}/{len}] {msg}")?
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );
        info.set_prefix("Handle");
        let process = multi.add(ProgressBar::new(len));
        process.enable_steady_tick(Duration::from_secs(1));
        process.set_style(
            ProgressStyle::with_template(
                "{prefix:>10.blue.bold} [{elapsed_precise}][{wide_bar}] ",
            )?
            .progress_chars("=> "),
        );
        process.set_prefix("Process");

        Ok(Bar {
            timer: Instant::now(),
            multi,
            info,
            process,
            success: 0,
            failed: 0,
            total: len as u32,
        })
    }

    pub fn new_check() -> anyhow::Result<ProgressBar> {
        let bar = ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(100));
        bar.set_style(
            ProgressStyle::with_template("{prefix:>10.blue.bold} {spinner} {msg}")?
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );
        bar.set_prefix("Check");

        Ok(bar)
    }

    pub fn message(&self, msg: &str) {
        info!("{msg}");
        self.info.set_message(msg.to_string());
    }

    pub fn info(&mut self, msg: &str) {
        self.success += 1;
        info!("{msg}");
        self.info.inc(1);
        self.process.inc(1);
        self.process
            .println(format!("{:>10} ✔️️ {}", style("Handle").green().bold(), msg));
    }

    pub fn println(&self, msg: &str) {
        info!("{msg}");
        self.process
            .println(format!("{:>10}️️ {}", style("Now").green().bold(), msg));
    }

    pub fn warn(&mut self, msg: &str) {
        self.failed += 1;
        warn!("{msg}");
        self.info.inc(1);
        self.process.inc(1);
        self.process
            .println(format!("{:>10} ✖️ {}", style("Handle").yellow().bold(), msg));
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        self.multi.clear().ok();
        println!(
            "{:>10} {}{}({}) {}({}) took {}",
            style("Finish").blue().bold(),
            if self.total == self.success {
                "🎉 "
            } else {
                ""
            },
            style("Success").green().bold(),
            self.success,
            style("Failed").yellow().bold(),
            self.failed,
            HumanDuration(self.timer.elapsed())
        );
        info!(
            "Success({}) Failed({}) took {}",
            self.success,
            self.failed,
            HumanDuration(self.timer.elapsed())
        );
    }
}
