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
}

#[tonic::async_trait]
impl data_service::data_service_server::DataService for DbService {
    async fn get_history(&self, request: Request<GetHistoryRequest>) -> Result<Response<GetHistoryResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        match self.user_memories.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let history_json = doc.get("history").map(|h| h.to_string()).unwrap_or_else(|| "[]".into());
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
                let data_json = mongodb::bson::to_document(&doc).map(|d| d.to_string()).unwrap_or_default();
                Ok(Response::new(GetUserStatResponse { data_json }))
            }
            _ => Ok(Response::new(GetUserStatResponse { data_json: "".into() })),
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
                let data_json = mongodb::bson::to_document(&doc).map(|d| d.to_string()).unwrap_or_default();
                Ok(Response::new(GetEconomyResponse { data_json }))
            }
            _ => Ok(Response::new(GetEconomyResponse { data_json: "".into() })),
        }
    }

    async fn upsert_economy(&self, request: Request<UpsertEconomyRequest>) -> Result<Response<UpsertEconomyResponse>, Status> {
        let req = request.into_inner();
        let data: Bson = serde_json::from_str(&req.data_json).map(|v: Value| to_bson(&v).unwrap_or(Bson::Null)).unwrap_or(Bson::Null);
        let filter = doc! { "user_id": &req.user_id };
        let update = doc! { "$set": data };
        let opts = UpdateOptions::builder().upsert(true).build();
        self.economy_players.update_one(filter, update, opts).await.map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpsertEconomyResponse { success: true }))
    }

    async fn get_rpg(&self, request: Request<GetRpgRequest>) -> Result<Response<GetRpgResponse>, Status> {
        let req = request.into_inner();
        let filter = doc! { "user_id": &req.user_id };
        match self.rpg_players.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let data_json = mongodb::bson::to_document(&doc).map(|d| d.to_string()).unwrap_or_default();
                Ok(Response::new(GetRpgResponse { data_json }))
            }
            _ => Ok(Response::new(GetRpgResponse { data_json: "".into() })),
        }
    }

    async fn upsert_rpg(&self, request: Request<UpsertRpgRequest>) -> Result<Response<UpsertRpgResponse>, Status> {
        let req = request.into_inner();
        let data: Bson = serde_json::from_str(&req.data_json).map(|v: Value| to_bson(&v).unwrap_or(Bson::Null)).unwrap_or(Bson::Null);
        let filter = doc! { "user_id": &req.user_id };
        let update = doc! { "$set": data };
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
            if let Ok(d) = mongodb::bson::to_document(&doc) {
                items.push(d.to_string());
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
    };

    println!("..db-manager listening on {}", addr);
    Server::builder()
        .add_service(data_service::data_service_server::DataServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
