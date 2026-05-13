use bot_messaging::bot_messaging_server::{BotMessaging, BotMessagingServer};
use bot_messaging::*;
use data_service::data_service_client::DataServiceClient;
use serde_json::{json, Value};
use std::env;
use tonic::{transport::Channel, transport::Server, Request, Response, Status};

pub mod bot_messaging {
    tonic::include_proto!("bot_messaging");
}

pub mod data_service {
    tonic::include_proto!("data_service");
}

struct AiService {
    db: DataServiceClient<Channel>,
    http: reqwest::Client,
    groq_key: String,
}

const HISTORY_LIMIT: usize = 30;

fn build_messages(system_prompt: &str, history_json: &str, user_msg: &str) -> Vec<Value> {
    let mut messages = vec![json!({"role": "system", "content": system_prompt})];
    let history: Vec<Value> = serde_json::from_str(history_json).unwrap_or_default();
    let recent = if history.len() > 6 {
        &history[history.len() - 6..]
    } else {
        &history[..]
    };
    messages.extend(recent.iter().cloned());
    messages.push(json!({"role": "user", "content": user_msg}));
    messages
}

fn update_history_json(history_json: &str, user_msg: &str, reply: &str) -> String {
    let mut history: Vec<Value> =
        serde_json::from_str(history_json).unwrap_or_default();
    history.push(json!({"role": "user", "content": user_msg}));
    history.push(json!({"role": "assistant", "content": reply}));
    if history.len() > HISTORY_LIMIT {
        history = history[history.len() - HISTORY_LIMIT..].to_vec();
    }
    serde_json::to_string(&history).unwrap_or_else(|_| "[]".into())
}

#[tonic::async_trait]
impl BotMessaging for AiService {
    async fn chat(
        &self,
        request: Request<ChatRequest>,
    ) -> Result<Response<ChatResponse>, Status> {
        let req = request.into_inner();

        let messages = build_messages(&req.system_prompt, &req.history_json, &req.user_msg);

        let req_body = json!({
            "model": "llama-3.3-70b-versatile",
            "messages": messages,
            "temperature": 0.85
        });

        let reply = match self
            .http
            .post("https://api.groq.com/openai/v1/chat/completions")
            .bearer_auth(&self.groq_key)
            .json(&req_body)
            .send()
            .await
        {
            Ok(r) => match r.json::<Value>().await {
                Ok(val) => val["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("...")
                    .to_string(),
                Err(e) => {
                    eprintln!("Groq chat error: {}", e);
                    "พังว่ะ ระบบ Groq เอ๋อ!".to_string()
                }
            },
            Err(e) => {
                eprintln!("Groq request error: {}", e);
                "พังว่ะ ระบบ Groq เอ๋อ!".to_string()
            }
        };

        let new_history_json = update_history_json(&req.history_json, &req.user_msg, &reply);

        if !req.user_key.is_empty() {
            let _ = self
                .db
                .clone()
                .update_history(data_service::UpdateHistoryRequest {
                    user_id: req.user_key,
                    history_json: new_history_json.clone(),
                })
                .await;
        }

        Ok(Response::new(ChatResponse {
            reply,
            updated_history_json: new_history_json,
        }))
    }

    async fn analyze(
        &self,
        request: Request<AnalyzeRequest>,
    ) -> Result<Response<AnalyzeResponse>, Status> {
        let req = request.into_inner();

        let payload = json!({
            "model": "llama-3.1-8b-instant",
            "messages": [
                {"role": "system", "content": "Analyze user text. Output ONLY 2 numbers separated by comma: (Profanity 0-10, Lewdness 0-10). Example: 8,0. No text, no explanation."},
                {"role": "user", "content": req.text}
            ],
            "temperature": 0
        });

        let (rude, lewd) = match self
            .http
            .post("https://api.groq.com/openai/v1/chat/completions")
            .bearer_auth(&self.groq_key)
            .json(&payload)
            .send()
            .await
        {
            Ok(r) => match r.json::<Value>().await {
                Ok(val) => {
                    if let Some(content) = val["choices"][0]["message"]["content"].as_str() {
                        let cleaned: String =
                            content.chars().filter(|c| c.is_ascii_digit() || *c == ',').collect();
                        let parts: Vec<&str> = cleaned.split(',').collect();
                        if parts.len() == 2 {
                            let rude = parts[0].parse().unwrap_or(0);
                            let lewd = parts[1].parse().unwrap_or(0);
                            (rude, lewd)
                        } else {
                            (0, 0)
                        }
                    } else {
                        (0, 0)
                    }
                }
                Err(e) => {
                    eprintln!("Groq analyze error: {}", e);
                    (0, 0)
                }
            },
            Err(e) => {
                eprintln!("Groq request error: {}", e);
                (0, 0)
            }
        };

        Ok(Response::new(AnalyzeResponse { rude, lewd }))
    }

    async fn summarize_gossip(
        &self,
        request: Request<SummarizeGossipRequest>,
    ) -> Result<Response<SummarizeGossipResponse>, Status> {
        let req = request.into_inner();

        let summary_prompt = format!(
            "จากข้อมูลเดิม: {}\nและข้อความใหม่: \"{}\"\nสรุปนิสัยคนนี้ไม่เกิน 1 ประโยค",
            req.old_summary, req.new_msg
        );

        let messages = vec![
            json!({"role": "system", "content": "สรุปนิสัยคนจากข้อความ ตอบสั้นๆ ไม่เกิน 1 ประโยค"}),
            json!({"role": "user", "content": summary_prompt}),
        ];

        let req_body = json!({
            "model": "llama-3.3-70b-versatile",
            "messages": messages,
            "temperature": 0.85
        });

        let summary = match self
            .http
            .post("https://api.groq.com/openai/v1/chat/completions")
            .bearer_auth(&self.groq_key)
            .json(&req_body)
            .send()
            .await
        {
            Ok(r) => match r.json::<Value>().await {
                Ok(val) => val["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("...")
                    .to_string(),
                Err(e) => {
                    eprintln!("Groq gossip error: {}", e);
                    "...".into()
                }
            },
            Err(e) => {
                eprintln!("Groq request error: {}", e);
                "...".into()
            }
        };

        if !req.user_id.is_empty() {
            let _ = self
                .db
                .clone()
                .update_gossip(data_service::UpdateGossipRequest {
                    user_id: req.user_id.clone(),
                    username: req.username.clone(),
                    summary: summary.clone(),
                })
                .await;
        }

        Ok(Response::new(SummarizeGossipResponse { summary }))
    }

    // --- Proxy methods: accept ai_core::* types, convert to data_service::* internally ---

    async fn get_history(
        &self,
        request: Request<GetHistoryRequest>,
    ) -> Result<Response<GetHistoryResponse>, Status> {
        let req = request.into_inner();
        let resp = self.db.clone().get_history(data_service::GetHistoryRequest { user_id: req.user_id }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        let inner = resp.into_inner();
        Ok(Response::new(GetHistoryResponse { history_json: inner.history_json }))
    }

    async fn update_history(
        &self,
        request: Request<UpdateHistoryRequest>,
    ) -> Result<Response<UpdateHistoryResponse>, Status> {
        let req = request.into_inner();
        self.db.clone().update_history(data_service::UpdateHistoryRequest {
            user_id: req.user_id,
            history_json: req.history_json,
        }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        Ok(Response::new(UpdateHistoryResponse { success: true }))
    }

    async fn get_economy(
        &self,
        request: Request<GetEconomyRequest>,
    ) -> Result<Response<GetEconomyResponse>, Status> {
        let req = request.into_inner();
        let resp = self.db.clone().get_economy(data_service::GetEconomyRequest { user_id: req.user_id }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        let inner = resp.into_inner();
        Ok(Response::new(GetEconomyResponse { data_json: inner.data_json }))
    }

    async fn upsert_economy(
        &self,
        request: Request<UpsertEconomyRequest>,
    ) -> Result<Response<UpsertEconomyResponse>, Status> {
        let req = request.into_inner();
        self.db.clone().upsert_economy(data_service::UpsertEconomyRequest {
            user_id: req.user_id,
            username: req.username,
            data_json: req.data_json,
        }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        Ok(Response::new(UpsertEconomyResponse { success: true }))
    }

    async fn get_rpg(
        &self,
        request: Request<GetRpgRequest>,
    ) -> Result<Response<GetRpgResponse>, Status> {
        let req = request.into_inner();
        let resp = self.db.clone().get_rpg(data_service::GetRpgRequest { user_id: req.user_id }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        let inner = resp.into_inner();
        Ok(Response::new(GetRpgResponse { data_json: inner.data_json }))
    }

    async fn upsert_rpg(
        &self,
        request: Request<UpsertRpgRequest>,
    ) -> Result<Response<UpsertRpgResponse>, Status> {
        let req = request.into_inner();
        self.db.clone().upsert_rpg(data_service::UpsertRpgRequest {
            user_id: req.user_id,
            username: req.username,
            class: req.class,
            data_json: req.data_json,
        }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        Ok(Response::new(UpsertRpgResponse { success: true }))
    }

    async fn find_all(
        &self,
        request: Request<FindAllRequest>,
    ) -> Result<Response<FindAllResponse>, Status> {
        let req = request.into_inner();
        let resp = self.db.clone().find_all(data_service::FindAllRequest { collection: req.collection }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        let inner = resp.into_inner();
        Ok(Response::new(FindAllResponse { items_json: inner.items_json }))
    }

    async fn update_user_stat(
        &self,
        request: Request<UpdateUserStatRequest>,
    ) -> Result<Response<UpdateUserStatResponse>, Status> {
        let req = request.into_inner();
        self.db.clone().update_user_stat(data_service::UpdateUserStatRequest {
            user_id: req.user_id,
            username: req.username,
            message_count_inc: req.message_count_inc,
            rude_score_inc: req.rude_score_inc,
            lewd_score_inc: req.lewd_score_inc,
        }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        Ok(Response::new(UpdateUserStatResponse { success: true }))
    }

    async fn narrate(
        &self,
        request: Request<NarrateRequest>,
    ) -> Result<Response<NarrateResponse>, Status> {
        let req = request.into_inner();
        let body = serde_json::json!({
            "model": "llama-3.3-70b-versatile",
            "messages": [{"role": "system", "content": req.system_prompt}],
            "temperature": 0.85
        });
        let text = match self.http
            .post("https://api.groq.com/openai/v1/chat/completions")
            .bearer_auth(&self.groq_key)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => match r.json::<serde_json::Value>().await {
                Ok(v) => v["choices"][0]["message"]["content"].as_str().unwrap_or("...").to_string(),
                Err(_) => "...".to_string(),
            },
            Err(_) => "...".to_string(),
        };
        Ok(Response::new(NarrateResponse { text }))
    }

    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        let req = request.into_inner();
        let resp = self.db.clone().get_config(data_service::GetConfigRequest { key: req.key }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        let inner = resp.into_inner();
        Ok(Response::new(GetConfigResponse { value: inner.value }))
    }

    async fn set_config(
        &self,
        request: Request<SetConfigRequest>,
    ) -> Result<Response<SetConfigResponse>, Status> {
        let req = request.into_inner();
        self.db.clone().set_config(data_service::SetConfigRequest {
            key: req.key,
            value: req.value,
        }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        Ok(Response::new(SetConfigResponse { success: true }))
    }

    async fn insert_web_command(
        &self,
        request: Request<InsertWebCommandRequest>,
    ) -> Result<Response<InsertWebCommandResponse>, Status> {
        let req = request.into_inner();
        self.db.clone().insert_web_command(data_service::InsertWebCommandRequest {
            command_id: req.command_id,
            command_type: req.command_type,
            payload_json: req.payload_json,
            status: req.status,
            created_at: req.created_at,
        }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        Ok(Response::new(InsertWebCommandResponse { success: true }))
    }

    async fn get_pending_web_commands(
        &self,
        _request: Request<GetPendingWebCommandsRequest>,
    ) -> Result<Response<GetPendingWebCommandsResponse>, Status> {
        let resp = self.db.clone().get_pending_web_commands(data_service::GetPendingWebCommandsRequest {}).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        let inner = resp.into_inner();
        Ok(Response::new(GetPendingWebCommandsResponse { items_json: inner.items_json }))
    }

    async fn update_web_command_status(
        &self,
        request: Request<UpdateWebCommandStatusRequest>,
    ) -> Result<Response<UpdateWebCommandStatusResponse>, Status> {
        let req = request.into_inner();
        self.db.clone().update_web_command_status(data_service::UpdateWebCommandStatusRequest {
            command_id: req.command_id,
            status: req.status,
        }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        Ok(Response::new(UpdateWebCommandStatusResponse { success: true }))
    }

    async fn delete_web_command(
        &self,
        request: Request<DeleteWebCommandRequest>,
    ) -> Result<Response<DeleteWebCommandResponse>, Status> {
        let req = request.into_inner();
        self.db.clone().delete_web_command(data_service::DeleteWebCommandRequest {
            command_id: req.command_id,
        }).await
            .map_err(|e| Status::internal(format!("db-manager error: {}", e)))?;
        Ok(Response::new(DeleteWebCommandResponse { success: true }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let addr = env::var("AI_CORE_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50052".into())
        .parse()?;

    let db_addr = env::var("DATA_SERVICE_ADDR")
        .unwrap_or_else(|_| "http://0.0.0.0:50051".into());
    let db = DataServiceClient::connect(db_addr).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to data-service: {}", e))?;

    let groq_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");

    println!("🤖 ai-core listening on {}", addr);

    let service = AiService {
        db,
        http: reqwest::Client::new(),
        groq_key,
    };

    Server::builder()
        .add_service(BotMessagingServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
