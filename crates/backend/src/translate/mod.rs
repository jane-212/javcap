use std::collections::BTreeMap;

use async_trait::async_trait;

#[async_trait]
pub trait Translator {
    async fn translate<'a>(&self, text: BTreeMap<&'a str, String>) -> BTreeMap<&'a str, String>;
}
