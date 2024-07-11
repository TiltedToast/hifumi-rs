use std::collections::HashMap;

use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use mongodb::Client as MongoClient;
use serde::{Deserialize, Serialize};
use serenity::{model::prelude::Message, prelude::Context};
use tokio::sync::RwLock;

use crate::config::Config;

pub type StatusVec = RwLock<Vec<StatusDoc>>;
pub type PrefixMap = RwLock<HashMap<String, String>>;

pub struct MessageCommandData<'a> {
    pub ctx: &'a Context,
    pub msg: &'a Message,
    pub content: Vec<String>,
    pub command: String,
    pub react_cmd: Option<String>,
    pub sub_cmd: Option<String>,
    pub handler: &'a Handler<'a>,
    pub prefix: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusDoc {
    pub _id: ObjectId,
    pub r#type: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrefixDoc {
    pub _id: ObjectId,
    #[serde(rename = "serverId")]
    pub server_id: String,
    pub prefix: String,
}

/// Handler contains the data necessary to run the bot. This includes the start
/// time, the configuration, the database client, the statuses, and the
/// prefixes.
pub struct Handler<'a> {
    pub start_time: DateTime<Utc>,
    pub config: Config<'a>,
    pub db_client: MongoClient,
    pub statuses: StatusVec,
    pub prefixes: PrefixMap,
}
