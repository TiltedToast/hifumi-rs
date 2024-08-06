use std::collections::HashMap;
use crate::db::models::Status;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::{all::UserId, model::prelude::Message, prelude::Context};
use tokio::sync::RwLock;

use crate::config::Config;

pub type StatusVec = RwLock<Vec<Status>>;
pub type PrefixMap = RwLock<HashMap<String, String>>;

#[allow(dead_code)]
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
pub struct Owners {
    pub primary: UserId,
    pub secondary: Vec<UserId>,
}

/// Handler contains the data necessary to run the bot. This includes the start
/// time, the configuration, the database client, the statuses, and the
/// prefixes.
pub struct Handler<'a> {
    pub start_time: DateTime<Utc>,
    pub config: Config<'a>,
    pub db_pool: sqlx::SqlitePool,
    pub statuses: StatusVec,
    pub prefixes: PrefixMap,
}
