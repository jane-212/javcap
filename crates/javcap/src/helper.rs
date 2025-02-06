use anyhow::Result;
use config::Translator as CfgTranslator;
use spider::Spider;
use tokio::sync::Semaphore;
use translator::Translator;

pub struct Helper {
    pub sema: Semaphore,
    pub spider: Spider,
    pub translator: Translator,
}

impl Helper {
    pub fn new(
        task_limit: usize,
        translators: &[CfgTranslator],
        timeout: u64,
        proxy: Option<String>,
    ) -> Result<Helper> {
        let sema = Semaphore::new(task_limit);
        let spider = Spider::new(timeout, proxy.clone())?;
        let translator = Translator::new(translators, timeout, proxy)?;
        let helper = Helper {
            sema,
            spider,
            translator,
        };

        Ok(helper)
    }
}
