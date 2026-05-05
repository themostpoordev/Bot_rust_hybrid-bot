use bot_messaging::bot_messaging_client::BotMessagingClient;
use bot_messaging::*;
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use tonic::transport::Channel;

pub mod bot_messaging {
    tonic::include_proto!("bot_messaging");
}

// ---------------------------------------------------------------------------
// LINE Webhook Types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct LineEvent {
    events: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// App State
// ---------------------------------------------------------------------------

struct AppState {
    ai: BotMessagingClient<Channel>,
    http: reqwest::Client,
    line_token: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn line_webhook(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LineEvent>,
) -> &'static str {
    println!("📥 [LINE] มีข้อความวิ่งเข้า Webhook!");

    for event in payload.events {
        let Some(reply_token) = event["replyToken"].as_str() else { continue; };
        let Some(text) = event["message"]["text"].as_str() else { continue; };
        println!("💬 [LINE] User พิมพ์มาว่า: {}", text);

        let user_id = event["source"]["userId"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let line_user_key = format!("line_{}", user_id);

        if text == "!ลืม" {
            let _ = state
                .ai
                .clone()
                .update_history(UpdateHistoryRequest {
                    user_id: line_user_key.clone(),
                    history_json: "[]".into(),
                })
                .await;
            let _ = state
                .http
                .post("https://api.line.me/v2/bot/message/reply")
                .bearer_auth(&state.line_token)
                .json(&json!({ "replyToken": reply_token, "messages": [{"type": "text", "text": "ล้างสมองกูหาพ่อมึงเหรอ!"}] }))
                .send()
                .await;
            continue;
        }

        // Get history from ai-core (which proxies to db-manager)
        let history_json = match state
            .ai
            .clone()
            .get_history(GetHistoryRequest {
                user_id: line_user_key.clone(),
            })
            .await
        {
            Ok(r) => r.into_inner().history_json,
            Err(_) => "[]".into(),
        };

        let response = state
            .ai
            .clone()
            .chat(ChatRequest {
                user_msg: text.into(),
                history_json,
                system_prompt: "มึงชื่อ 'ต๊ะ' เป็น AI วัยรุ่นทรงเอ ปากหมา กวนตีน".into(),
                user_key: line_user_key.clone(),
            })
            .await;

        let reply = match response {
            Ok(r) => r.into_inner().reply,
            Err(_) => "พังว่ะ!".into(),
        };

        println!("🤖 [LINE] ตอบ: {}", reply);

        let _ = state
            .http
            .post("https://api.line.me/v2/bot/message/push")
            .bearer_auth(&state.line_token)
            .json(&json!({ "to": user_id, "messages": [{"type": "text", "text": reply}] }))
            .send()
            .await;
    }

    "OK"
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let ai_addr = env::var("BOT_MESSAGING_ADDR")
        .unwrap_or_else(|_| "http://0.0.0.0:50052".into());
    let ai = BotMessagingClient::connect(ai_addr).await?;

    let line_token = env::var("LINE_TOKEN").expect("LINE_TOKEN not set");
    let port = env::var("LINE_PORT").unwrap_or_else(|_| "8080".into());
    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

    let state = Arc::new(AppState {
        ai,
        http: reqwest::Client::new(),
        line_token,
    });

    let app = Router::new()
        .route("/webhook", post(line_webhook))
        .with_state(state);

    println!("🚀 gateway-line listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
