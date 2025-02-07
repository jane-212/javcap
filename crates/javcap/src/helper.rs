use anyhow::Result;
use config::Config;
use spider::Spider;
use tokio::sync::Semaphore;
use translator::Translator;

pub struct Helper {
    pub sema: Semaphore,
    pub spider: Spider,
    pub translator: Translator,
}

impl Helper {
    pub fn new(config: &Config) -> Result<Helper> {
        let sema = Semaphore::new(config.task_limit);
        let spider = Spider::new(config)?;
        let translator = Translator::new(config)?;
        let helper = Helper {
            sema,
            spider,
            translator,
        };

        Ok(helper)
    }
}
