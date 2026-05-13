use data_service::*;
use mongodb::{
    bson::{doc, to_bson, Bson},
    options::UpdateOptions,
    Client as MongoClient, Collection,
};
use futures::stream::TryStreamExt;
use serde_json::Value;
use std::env;
use tonic::{transport::Server, Request, Response, Status};

pub mod data_service {
    tonic::include_proto!("data_service");
}

struct DbService {
    user_memories: Collection<Value>,
    user_stats: Collection<Bson>,
    user_gossip: Collection<Bson>,
    economy_players: Collection<Bson>,
    rpg_players: Collection<Bson>,
    config: Collection<Bson>,
    web_commands: Collection<Bson>,
}

#[tonic::async_trait]
impl data_service::data_service_server::DataService for DbService {
    async fn get_history(&self, request: Request<GetHistoryRequest>) -> Result<Response<GetHistoryResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        match self.user_memories.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let history_json = if let Some(h) = doc.get("history") {
                    if let Ok(val) = serde_json::to_value(h) {
                        serde_json::to_string(&val).unwrap_or_else(|_| "[]".into())
                    } else {
                        "[]".into()
                    }
                } else {
                    "[]".into()
                };
                Ok(Response::new(GetHistoryResponse { history_json }))
            }
            _ => Ok(Response::new(GetHistoryResponse { history_json: "[]".into() })),
        }
    }

    async fn update_history(&self, request: Request<UpdateHistoryRequest>) -> Result<Response<UpdateHistoryResponse>, Status> {
        let req = request.into_inner();
        let history_json: Value = serde_json::from_str(&req.history_json).unwrap_or(serde_json::json!([]));
        let bson_history = to_bson(&history_json).map_err(|e| Status::internal(e.to_string()))?;
        let filter = doc! { "user_id": &req.user_id };
        let update = doc! { "$set": { "history": bson_history } };
        let opts = UpdateOptions::builder().upsert(true).build();
        self.user_memories.update_one(filter, update, opts).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateHistoryResponse { success: true }))
    }

    async fn delete_history(&self, request: Request<DeleteHistoryRequest>) -> Result<Response<DeleteHistoryResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        self.user_memories.delete_one(filter, None).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteHistoryResponse { success: true }))
    }

    async fn get_gossip(&self, request: Request<GetGossipRequest>) -> Result<Response<GetGossipResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        match self.user_gossip.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let summary = doc.as_document().and_then(|d| d.get_str("summary").ok()).unwrap_or_default().to_string();
                let username = doc.as_document().and_then(|d| d.get_str("username").ok()).unwrap_or_default().to_string();
                Ok(Response::new(GetGossipResponse { summary, username }))
            }
            _ => Ok(Response::new(GetGossipResponse { summary: "".into(), username: "".into() })),
        }
    }

    async fn update_gossip(&self, request: Request<UpdateGossipRequest>) -> Result<Response<UpdateGossipResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        let update = doc! { "$set": { "username": &req.username, "summary": &req.summary } };
        let opts = UpdateOptions::builder().upsert(true).build();
        self.user_gossip.update_one(filter, update, opts).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateGossipResponse { success: true }))
    }

    async fn get_user_stat(&self, request: Request<GetUserStatRequest>) -> Result<Response<GetUserStatResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        match self.user_stats.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let mut doc_bson = mongodb::bson::to_document(&doc).map_err(|e| Status::internal(e.to_string()))?;
                doc_bson.remove("_id");
                let data_json = serde_json::to_value(&doc_bson).ok().and_then(|v| serde_json::to_string(&v).ok()).unwrap_or_else(|| "{}".into());
                Ok(Response::new(GetUserStatResponse { data_json }))
            }
            _ => Ok(Response::new(GetUserStatResponse { data_json: "{}".into() })),
        }
    }

    async fn update_user_stat(&self, request: Request<UpdateUserStatRequest>) -> Result<Response<UpdateUserStatResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        let update = doc! {
            "$set": { "username": &req.username },
            "$inc": {
                "message_count": req.message_count_inc as i64,
                "rude_score": req.rude_score_inc as i64,
                "lewd_score": req.lewd_score_inc as i64
            }
        };
        let opts = UpdateOptions::builder().upsert(true).build();
        self.user_stats.update_one(filter, update, opts).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateUserStatResponse { success: true }))
    }

    async fn get_economy(&self, request: Request<GetEconomyRequest>) -> Result<Response<GetEconomyResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        match self.economy_players.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let mut doc_bson = mongodb::bson::to_document(&doc).map_err(|e| Status::internal(e.to_string()))?;
                doc_bson.remove("_id");
                let data_json = serde_json::to_value(&doc_bson).ok().and_then(|v| serde_json::to_string(&v).ok()).unwrap_or_else(|| "{}".into());
                Ok(Response::new(GetEconomyResponse { data_json }))
            }
            _ => Ok(Response::new(GetEconomyResponse { data_json: "{}".into() })),
        }
    }

    async fn upsert_economy(&self, request: Request<UpsertEconomyRequest>) -> Result<Response<UpsertEconomyResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        let mut set_doc = doc! { "username": &req.username };
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&req.data_json) {
            if let Ok(bson_val) = to_bson(&value) {
                if let Bson::Document(mut d) = bson_val {
                    d.insert("username", Bson::String(req.username.clone()));
                    set_doc = d;
                }
            }
        }
        let update = doc! { "$set": set_doc };
        let opts = UpdateOptions::builder().upsert(true).build();
        self.economy_players.update_one(filter, update, opts).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpsertEconomyResponse { success: true }))
    }

    async fn get_rpg(&self, request: Request<GetRpgRequest>) -> Result<Response<GetRpgResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        match self.rpg_players.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let mut doc_bson = mongodb::bson::to_document(&doc).map_err(|e| Status::internal(e.to_string()))?;
                doc_bson.remove("_id");
                let data_json = serde_json::to_value(&doc_bson).ok().and_then(|v| serde_json::to_string(&v).ok()).unwrap_or_else(|| "{}".into());
                Ok(Response::new(GetRpgResponse { data_json }))
            }
            _ => Ok(Response::new(GetRpgResponse { data_json: "{}".into() })),
        }
    }

    async fn upsert_rpg(&self, request: Request<UpsertRpgRequest>) -> Result<Response<UpsertRpgResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        let mut set_doc = doc! { "username": &req.username };
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&req.data_json) {
            if let Ok(bson_val) = to_bson(&value) {
                if let Bson::Document(mut d) = bson_val {
                    d.insert("username", Bson::String(req.username.clone()));
                    set_doc = d;
                }
            }
        }
        let update = doc! { "$set": set_doc };
        let opts = UpdateOptions::builder().upsert(true).build();
        self.rpg_players.update_one(filter, update, opts).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpsertRpgResponse { success: true }))
    }

    async fn get_config(&self, request: Request<GetConfigRequest>) -> Result<Response<GetConfigResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "key": &req.key };
        match self.config.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let value = doc.as_document().and_then(|d| d.get_str("value").ok()).unwrap_or_default().to_string();
                Ok(Response::new(GetConfigResponse { value }))
            }
            _ => Ok(Response::new(GetConfigResponse { value: "".into() })),
        }
    }

    async fn set_config(&self, request: Request<SetConfigRequest>) -> Result<Response<SetConfigResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "key": &req.key };
        let update = doc! { "$set": { "value": &req.value } };
        let opts = UpdateOptions::builder().upsert(true).build();
        self.config.update_one(filter, update, opts).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SetConfigResponse { success: true }))
    }

    async fn insert_web_command(&self, request: Request<InsertWebCommandRequest>) -> Result<Response<InsertWebCommandResponse>, Status> {
        let req = request.into_inner();
        let doc = doc! {
            "command_id": &req.command_id,
            "command_type": &req.command_type,
            "payload_json": &req.payload_json,
            "status": &req.status,
            "created_at": req.created_at as i64,
        };
        let bson_doc = to_bson(&doc).map_err(|e| Status::internal(e.to_string()))?;
        self.web_commands.insert_one(bson_doc, None).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(InsertWebCommandResponse { success: true }))
    }

    async fn get_pending_web_commands(&self, _request: Request<GetPendingWebCommandsRequest>) -> Result<Response<GetPendingWebCommandsResponse>, Status> {
        let filter = doc! { "status": "pending" };
        let sort = doc! { "created_at": 1 };
        let find_options = mongodb::options::FindOptions::builder().sort(sort).build();
        let mut cursor = self.web_commands.find(filter, find_options).await.map_err(|e| Status::internal(e.to_string()))?;
        let mut items = vec![];
        while let Some(doc) = cursor.try_next().await.map_err(|e| Status::internal(e.to_string()))? {
            if let Ok(mut d) = mongodb::bson::to_document(&doc) {
                d.remove("_id");
                if let Ok(json_val) = serde_json::to_value(&d) {
                    items.push(json_val.to_string());
                }
            }
        }
        Ok(Response::new(GetPendingWebCommandsResponse { items_json: items }))
    }

    async fn update_web_command_status(&self, request: Request<UpdateWebCommandStatusRequest>) -> Result<Response<UpdateWebCommandStatusResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "command_id": &req.command_id };
        let update = doc! { "$set": { "status": &req.status } };
        self.web_commands.update_one(filter, update, None).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateWebCommandStatusResponse { success: true }))
    }

    async fn delete_web_command(&self, request: Request<DeleteWebCommandRequest>) -> Result<Response<DeleteWebCommandResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "command_id": &req.command_id };
        self.web_commands.delete_one(filter, None).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteWebCommandResponse { success: true }))
    }

    async fn find_all(&self, request: Request<FindAllRequest>) -> Result<Response<FindAllResponse>, Status> {
        let req = request.into_inner();
        let collection: &Collection<Bson> = match req.collection.as_str() {
            "economy" => &self.economy_players,
            "rpg" => &self.rpg_players,
            "stats" => &self.user_stats,
            _ => { return Ok(Response::new(FindAllResponse { items_json: vec![] })) }
        };
        let mut cursor = collection.find(None, None).await.map_err(|e| Status::internal(e.to_string()))?;
        let mut items = vec![];
        while let Some(doc) = cursor.try_next().await.map_err(|e| Status::internal(e.to_string()))? {
            if let Ok(mut d) = mongodb::bson::to_document(&doc) {
                d.remove("_id");
                if let Ok(json_val) = serde_json::to_value(&d) {
                    items.push(json_val.to_string());
                }
            }
        }
        Ok(Response::new(FindAllResponse { items_json: items }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let addr = env::var("DB_MANAGER_ADDR").unwrap_or_else(|_| "0.0.0.0:50051".into()).parse()?;

    let mongo_uri = env::var("MONGODB_URI").expect("MONGODB_URI not set");
    let client = MongoClient::with_uri_str(&mongo_uri).await?;

    let service = DbService {
        user_memories: client.database("tee_bot_db").collection("user_memories"),
        user_stats: client.database("poordev_db").collection("user_stats"),
        user_gossip: client.database("poordev_db").collection("user_gossip"),
        economy_players: client.database("tah_economy").collection("economy_players"),
        rpg_players: client.database("tah_rpg").collection("rpg_players"),
        config: client.database("tah_config").collection("config"),
        web_commands: client.database("tah_config").collection("web_commands"),
    };

    println!("..db-manager listening on {}", addr);
    Server::builder()
        .add_service(data_service::data_service_server::DataServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
