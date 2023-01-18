use crate::config::Config;
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use mongodb::Client as MongoClient;
use serde::{Deserialize, Serialize};
use serenity::{model::prelude::Message, prelude::Context};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub type StatusVec = RwLock<Vec<StatusDoc>>;
pub type PrefixMap = RwLock<HashMap<String, String>>;

pub struct MessageCommandData<'a> {
    pub ctx: &'a Context,
    pub msg: &'a Message,
    pub content: Vec<String>,
    pub command: String,
    pub react_cmd: String,
    pub sub_cmd: String,
    pub handler: &'a Handler,
    pub prefix: String,
}

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

/// Handler contains the data necessary to run the bot. This includes the start time,
/// the configuration, the database client, the statuses, and the prefixes.
pub struct Handler {
    pub start_time: DateTime<Utc>,
    pub config: Config,
    pub db_client: MongoClient,
    pub statuses: StatusVec,
    pub prefixes: PrefixMap,
}
