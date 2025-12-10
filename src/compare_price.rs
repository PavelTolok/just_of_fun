use crate::{share_state::SharedState, telegram::send_telegram_message};
use chrono::Utc;
use std::{error, sync::Arc};

pub async fn compare_prices(
    shared_state: &Arc<SharedState>,
    symbol: &str,
) -> Result<(), Box<dyn error::Error>> {
    let bybit_price = {
        let bybit_prices = shared_state.bybit_prices.read().await;
        *bybit_prices.get(symbol).unwrap_or(&0.0)
    };

    let hyperliquid_price = {
        let hyperliquid_prices = shared_state.hyperliquid_prices.read().await;
        *hyperliquid_prices.get(symbol).unwrap_or(&0.0)
    };

    if bybit_price == 0.0 || hyperliquid_price == 0.0 {
        return Ok(());
    }

    let difference = ((bybit_price - hyperliquid_price) / bybit_price).abs() * 100.0;
    // println!(
    //     "comparing: {:#?}, bybit price: {:#?}, hyperliquid price: {:#?}, difference: {:.5}%",
    //     symbol, bybit_price, hyperliquid_price, difference
    // );

    // HERE

    if difference >= 5.0 {
        let tweeted_symbols = shared_state.tweeted_symbols.read().await;
        if !tweeted_symbols.contains(symbol) {
            drop(tweeted_symbols);

            let message_text = format!(
                "5% difference for {}:\nBYBIT price: {}, HYPERLIQUID price: {}\nDIFFERENCE: {:.5}%\nTimestamp: {}",
                symbol, bybit_price, hyperliquid_price, difference, Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
            );
            println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n{}\n@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@", message_text);
            println!(
                ">5%: {:#?}, bybit price: {:#?}, hyperliquid price: {:#?}, difference: {:.5}%",
                symbol, bybit_price, hyperliquid_price, difference
            );
            if let Err(e) = send_telegram_message(&message_text).await {
                eprintln!("Ошибка при отправке сообщения в Telegram: {}", e);
                // Продолжаем работу даже если не удалось отправить сообщение
            }

            let mut tweeted_symbols = shared_state.tweeted_symbols.write().await;
            tweeted_symbols.insert(symbol.to_string());
        }
    }

    // if symbol == "BTCUSDT" {
    //     let tweeted_symbols = shared_state.tweeted_symbols.read().await;
    //     if !tweeted_symbols.contains(symbol) {
    //         drop(tweeted_symbols);

    //         let tweet_text = format!(
    //             "$BTC BTCUSDT:\nBYBIT price: {}, HYPERLIQUID price: {}, difference: {:.5}%\nTimestamp: {}",
    //             bybit_price, hyperliquid_price, difference, Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    //             );
    //         println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@\n{}\n@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@", tweet_text);
    //         create_tweet(&tweet_text).await?;

    //         let mut tweeted_symbols = shared_state.tweeted_symbols.write().await;
    //         tweeted_symbols.insert(symbol.to_string());
    //     }
    // }

    Ok(())
}
