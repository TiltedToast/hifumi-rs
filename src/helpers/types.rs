use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use crate::config::Config;
use mongodb::Client as MongoClient;
use std::collections::HashMap;
use tokio::sync::Mutex;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusDoc {
    pub _id: ObjectId,
    pub r#type: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PrefixDoc {
    pub _id: ObjectId,
    pub serverId: String,
    pub prefix: String,
}

#[allow(dead_code)]
pub struct Handler {
    pub start_time: DateTime<Utc>,
    pub config: Config,
    pub db_client: MongoClient,
    pub statuses: Mutex<Vec<StatusDoc>>,
    pub prefixes: Mutex<HashMap<String, String>>,
}