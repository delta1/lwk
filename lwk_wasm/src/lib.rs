use lwk_wollet::{ElementsNetwork, EsploraWasmClient, NoPersist, Wollet, WolletDescriptor};
use std::{fmt::Debug, str::FromStr, sync::Arc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn balance() -> Result<JsValue, String> {
    let descriptor = WolletDescriptor::from_str("ct(slip77(ab5824f4477b4ebb00a132adfd8eb0b7935cf24f6ac151add5d1913db374ce92),elwpkh([759db348/84'/1'/0']tpubDCRMaF33e44pcJj534LXVhFbHibPbJ5vuLhSSPFAw57kYURv4tzXFL6LSnd78bkjqdmE3USedkbpXJUPA1tdzKfuYSL7PianceqAhwL2UkA/<0;1>/*))#cch6wrnp").map_err(to_debug)?;
    wasm_bindgen_test::console_log!("going to sync {}", descriptor);

    let network = ElementsNetwork::LiquidTestnet;
    let mut wollet = Wollet::new(network, Arc::new(NoPersist {}), descriptor).map_err(to_debug)?;

    let mut client = EsploraWasmClient::new("https://blockstream.info/liquidtestnet/api");

    let update = client.full_scan(&wollet).await.map_err(to_debug)?;
    if let Some(update) = update {
        wollet.apply_update(update).map_err(to_debug)?;
    }
    let balance = wollet.balance().map_err(to_debug)?;
    wasm_bindgen_test::console_log!("balance {:?}", balance);

    Ok(serde_wasm_bindgen::to_value(&balance).map_err(to_debug)?)
}

fn to_debug<D: Debug>(d: D) -> String {
    format!("{d:?}")
}

mod tests {
    use std::{collections::HashMap, str::FromStr};

    use lwk_wollet::elements::AssetId;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn balance_test() {
        let balance = crate::balance().await.unwrap();
        let balance: HashMap<AssetId, u64> = serde_wasm_bindgen::from_value(balance).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            AssetId::from_str("144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49")
                .unwrap(),
            84020,
        );
        assert_eq!(expected, balance);
    }
}