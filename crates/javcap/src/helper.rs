use anyhow::Result;
use spider::Spider;
use tokio::sync::Semaphore;
use translator::Translator;

pub struct Helper {
    pub sema: Semaphore,
    pub spider: Spider,
    pub translator: Translator,
}

impl Helper {
    pub fn new(task_limit: usize, timeout: u64, proxy: Option<String>) -> Result<Helper> {
        let sema = Semaphore::new(task_limit);
        let spider = Spider::new(timeout, proxy.clone())?;
        let translator = Translator::new(timeout, proxy)?;
        let helper = Helper {
            sema,
            spider,
            translator,
        };

        Ok(helper)
    }
}
