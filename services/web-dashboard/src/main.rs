use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Html,
    routing::{get, post},
    Router,
};
use bot_messaging::bot_messaging_client::BotMessagingClient;
use bot_messaging::FindAllRequest;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tonic::transport::Channel;

pub mod bot_messaging {
    tonic::include_proto!("bot_messaging");
}

mod dashboard;
mod admin;

#[derive(Debug, Deserialize)]
pub struct AdminCmd {
    pub bot_name: String,
    pub channel_id: String,
    pub role_id: String,
    pub max_people: usize,
    pub minutes: u64,
    pub pass: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AdminTask {
    pub bot_name: String,
    pub channel_id: u64,
    pub role_id: u64,
    pub max_people: usize,
    pub end_time: Instant,
    pub participants: Arc<tokio::sync::Mutex<HashSet<u64>>>,
}

pub struct AppState {
    pub ai: BotMessagingClient<Channel>,
    pub active_task: Arc<tokio::sync::Mutex<Option<AdminTask>>>,
}

fn get_host(headers: &HeaderMap) -> String {
    headers.get("host").and_then(|v| v.to_str().ok()).unwrap_or("").to_lowercase()
}

fn is_admin_host(host: &str) -> bool { host.starts_with("admin.") }

async fn root_handler(State(app): State<Arc<AppState>>, headers: HeaderMap, Query(params): Query<HashMap<String, String>>) -> Html<String> {
    if is_admin_host(&get_host(&headers)) {
        admin::admin_handler(State(app), headers, Query(params)).await
    } else {
        dashboard::dashboard_handler(State(app)).await
    }
}

async fn health_check() -> (StatusCode, String) {
    use std::time::Instant;

    let ai_addr = std::env::var("BOT_MESSAGING_ADDR")
        .unwrap_or_else(|_| "http://localhost:50052".into());

    // Check ai-core by attempting a gRPC connection
    let start = Instant::now();
    let (ai_status, ai_latency) = match BotMessagingClient::connect(ai_addr.clone()).await {
        Ok(mut client) => {
            // Verify it's actually working by calling find_all on a harmless collection
            match client.find_all(FindAllRequest { collection: "stats".into() }).await {
                Ok(_) => (true, start.elapsed().as_micros()),
                Err(_) => (false, 0),
            }
        }
        _ => (false, 0),
    };
    let ai_status_str = if ai_status { "online" } else { "offline" };
    let ai_latency_val = if ai_status { ai_latency } else { 0 };

    // If ai-core is online, db-manager is reachable (ai-core proxies through it)
    let db_status_str = ai_status_str;
    let db_latency_val = ai_latency_val;

    let entries = vec![
        format!(r#"{{"name":"ai-core","status":"{}","latency_us":{}}}"#, ai_status_str, ai_latency_val),
        format!(r#"{{"name":"db-manager","status":"{}","latency_us":{}}}"#, db_status_str, db_latency_val),
        r#"{"name":"web-dashboard","status":"online","latency_us":0}"#.to_string(),
    ];

    let body = format!(r#"{{"services":[{}]}}"#, entries.join(","));
    (StatusCode::OK, body)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let ai_addr = std::env::var("BOT_MESSAGING_ADDR").unwrap_or_else(|_| "http://localhost:50052".into());
    let ai = BotMessagingClient::connect(ai_addr).await?;
    let port = std::env::var("WEB_PORT").unwrap_or_else(|_| "8080".into());
    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    let state = Arc::new(AppState { ai, active_task: Arc::new(tokio::sync::Mutex::new(None)) });
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/cmd", post(admin::admin_post_handler))
        .route("/api/user/:id", get(admin::api_get_user))
        .route("/api/economy/set", post(admin::api_set_economy))
        .route("/api/rpg/set", post(admin::api_set_rpg))
        .route("/api/broadcast", post(admin::api_broadcast))
        .route("/api/announce", post(admin::api_announce))
        .route("/health", get(health_check))
        .with_state(state);
    println!("🌐 web-dashboard listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
