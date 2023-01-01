use crate::config::Config;
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use mongodb::Client as MongoClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;

pub type StatusVec = Mutex<Vec<StatusDoc>>;
pub type PrefixMap = Mutex<HashMap<String, String>>;

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

pub struct Handler {
    pub start_time: DateTime<Utc>,
    pub config: Config,
    pub db_client: MongoClient,
    pub statuses: StatusVec,
    pub prefixes: PrefixMap,
}
