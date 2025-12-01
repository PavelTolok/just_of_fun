use std::env;

use dotenv::dotenv;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct UpdateResponse {
    ok: bool,
    result: Vec<Update>,
}

#[derive(Debug, Deserialize)]
struct Update {
    message: Option<Message>,
}

#[derive(Debug, Deserialize)]
struct Message {
    chat: Chat,
}

#[derive(Debug, Deserialize)]
struct Chat {
    id: i64,
}

pub async fn log_all_chat_ids_from_telegram() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let bot_token = env::var("TELEGRAM_BOT_TOKEN")
        .map_err(|_| "TELEGRAM_BOT_TOKEN не установлен в переменных окружения")?;

    let url = format!("https://api.telegram.org/bot{}/getUpdates", bot_token);
    let client = Client::new();

    let resp = client.get(&url).send().await?;
    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        return Err(format!("Telegram getUpdates error: {}, body: {}", status, body).into());
    }

    let parsed: UpdateResponse = serde_json::from_str(&body)?;

    let mut ids = std::collections::HashSet::new();
    for u in parsed.result {
        if let Some(msg) = u.message {
            ids.insert(msg.chat.id);
        }
    }

    println!("TELEGRAM_CHAT_ID пользователей:");
    for id in ids {
        println!("{}", id);
    }

    Ok(())
}

pub async fn send_telegram_message(
    message_text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let bot_token = env::var("TELEGRAM_BOT_TOKEN")
        .map_err(|_| "TELEGRAM_BOT_TOKEN не установлен в переменных окружения")?;
    let chat_id = env::var("TELEGRAM_CHAT_ID")
        .map_err(|_| "TELEGRAM_CHAT_ID не установлен в переменных окружения")?;

    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    let client = Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "chat_id": chat_id,
            "text": message_text,
        }))
        .send()
        .await?;

    let status = resp.status();
    // let body = resp.text().await.unwrap_or_default();
    // println!("telegram response status: {}, body: {}", status, body);

    if !status.is_success() {
        return Err(format!("Telegram API error: {}", status).into());
    }

    Ok(())
}


