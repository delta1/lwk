use crate::{Error, Update, Wollet};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct EsploraClient {
    inner: lwk_wollet::EsploraWasmClient,
}

#[wasm_bindgen]
impl EsploraClient {
    /// Construct an Esplora Client
    pub fn new(url: &str) -> Self {
        let inner = lwk_wollet::EsploraWasmClient::new(url);
        Self { inner }
    }

    pub async fn full_scan(&mut self, wollet: &Wollet) -> Result<Option<Update>, Error> {
        let update: Option<lwk_wollet::Update> = self.inner.full_scan(wollet.as_ref()).await?;
        Ok(update.map(Into::into))
    }
}

mod tests {

    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_sleep() {
        lwk_wollet::sleep(1).await;
    }
}