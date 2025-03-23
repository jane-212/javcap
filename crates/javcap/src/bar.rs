use std::io::{self, IsTerminal, Write};
use std::sync::Arc;
use std::time::Duration;

use colored::Colorize;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::time;

pub struct Bar {
    cnt: Arc<RwLock<usize>>,
    total: Arc<Mutex<usize>>,
    should_quit: Arc<RwLock<bool>>,
    notify: Arc<Notify>,
    disabled: bool,
}

impl Bar {
    pub async fn new() -> Bar {
        let disabled = !io::stdout().is_terminal();
        let bar = Bar {
            total: Arc::new(Mutex::new(0)),
            cnt: Arc::new(RwLock::new(0)),
            should_quit: Arc::new(RwLock::new(false)),
            notify: Arc::new(Notify::new()),
            disabled,
        };
        bar.start().await;

        bar
    }

    pub async fn set_total(&self, total: usize) {
        let mut t = self.total.lock().await;
        *t = total;
    }

    async fn start(&self) {
        if self.disabled {
            return;
        }

        let should_quit = self.should_quit.clone();
        let notify = self.notify.clone();
        let cnt = self.cnt.clone();
        let total = self.total.clone();
        tokio::spawn(async move {
            let mut idx = 0;
            let interval = Duration::from_millis(200);
            let bar = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
            let bar_len = bar.len();
            let line_len = *app::LINE_LENGTH - 20;

            loop {
                let total = { *total.lock().await };
                let cnt = { *cnt.read().await };
                let per = if total == 0 { 0 } else { cnt * 100 / total };
                let p = per * line_len / 100;
                print!(
                    "\r{}",
                    format!(
                        "{spinner}|{per}%|{fill:░<line_len$}|[{cnt}/{total}]",
                        spinner = bar[idx],
                        fill = "█".repeat(p),
                        total = if total == 0 {
                            "?".to_string()
                        } else {
                            total.to_string()
                        }
                    )
                    .yellow()
                );
                io::stdout().flush().ok();
                idx += 1;
                idx %= bar_len;
                time::sleep(interval).await;
                if *should_quit.read().await {
                    break;
                }
            }
            notify.notify_one();
        });
    }

    pub async fn finish(&self) {
        if self.disabled {
            return;
        }

        {
            let mut should_quit = self.should_quit.write().await;
            *should_quit = true;
        }
        self.notify.notified().await;
        print!("\r{}\r", " ".repeat(*app::LINE_LENGTH));
    }

    pub fn message(&self, msg: impl AsRef<str>) {
        let msg = msg.as_ref();
        if self.disabled {
            println!("{msg}");
        } else {
            println!("\r{}\r{msg}", " ".repeat(*app::LINE_LENGTH));
        }
    }

    pub async fn add(&self) {
        let mut cnt = self.cnt.write().await;
        *cnt += 1;
    }
}
