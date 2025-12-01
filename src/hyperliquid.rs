use crate::share_state::SharedState;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;

pub struct HyperLiquidStruct {
    info_client: InfoClient,
}

#[derive(Debug, Deserialize)]
struct HyperliquidMetaResponse {
    universe: Vec<HyperliquidAssetMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HyperliquidAssetMeta {
    name: String,
    #[serde(default)]
    is_delisted: bool,
}

impl HyperLiquidStruct {
    pub async fn new() -> Self {
        Self {
            info_client: InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap(),
        }
    }

    fn format_ticker_name(ticker: &str) -> String {
        let formatted_ticker = if ticker.starts_with("k") {
            ticker.replacen("k", "1000", 1)
        } else {
            ticker.to_string()
        };
        format!("{}USDT", formatted_ticker)
    }

    pub async fn get_tickers(&self) -> Vec<String> {
        let request_body = json!({ "type": "meta" }).to_string();
        let response = self
            .info_client
            .http_client
            .post("/info", request_body)
            .await
            .expect("Failed to request Hyperliquid meta");
        let meta: HyperliquidMetaResponse =
            serde_json::from_str(&response).expect("Failed to parse Hyperliquid meta response");

        meta.universe
            .into_iter()
            .filter(|asset| !asset.is_delisted)
            .map(|asset| Self::format_ticker_name(&asset.name))
            .collect()
    }

    pub async fn hyperliquid_ws(mut self, shared_state: &Arc<SharedState>) {
        let (sender, mut receiver) = unbounded_channel();
        self.info_client
            .subscribe(Subscription::AllMids, sender)
            .await
            .unwrap();

        while let Some(Message::AllMids(all_mids)) = receiver.recv().await {
            for (ticker, price_str) in all_mids.data.mids.iter() {
                let formatted_ticker = Self::format_ticker_name(ticker);
                let price: f64 = price_str.parse().unwrap_or(0.0);
                {
                    let mut hyperliquid_prices = shared_state.hyperliquid_prices.write().await;
                    hyperliquid_prices.insert(formatted_ticker.clone(), price);
                }
                // let read_hyperliquid_prices = shared_state.hyperliquid_prices.read().await;
                // println!("share state: {:#?}", read_hyperliquid_prices);
                // compare_prices(shared_state, &formatted_ticker)
                //     .await
                //     .expect("Failed comparing price at hyperliquic");
            }
        }
    }
}


