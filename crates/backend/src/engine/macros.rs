#[macro_export]
macro_rules! image_loader {
    ($struct:ident) => {
        impl $struct {
            async fn load_img(&self, url: &str) -> anyhow::Result<Vec<u8>> {
                Ok(self.client.get(url).send().await?.bytes().await?.to_vec())
            }
        }
    };
}
