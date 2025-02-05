mod missav;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use missav::Missav;
use nfo::Nfo;

#[async_trait]
trait Finder: Send + Sync {
    async fn find(&self, key: &str) -> Result<Nfo>;
}

pub struct Spider {
    finders: Vec<Arc<dyn Finder>>,
}

impl Spider {
    pub fn new() -> Result<Spider> {
        let missav = Arc::new(Missav::new()?) as Arc<dyn Finder>;
        let finders = vec![missav];

        let spider = Spider { finders };
        Ok(spider)
    }

    pub async fn find(&self, key: &str) -> Result<Nfo> {
        let mut tasks = Vec::new();
        for finder in self.finders.iter() {
            let finder = finder.clone();
            let key = key.to_string();
            let task = tokio::spawn(async move { finder.find(&key).await });
            tasks.push(task);
        }

        let mut nfo = Nfo::new(key);
        nfo.set_mpaa("NC-17".to_string());
        for task in tasks {
            let found_nfo = task.await??;
            nfo.merge(found_nfo);
        }

        Ok(nfo)
    }
}
