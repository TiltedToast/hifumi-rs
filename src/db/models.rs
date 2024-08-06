use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct ErrorLog {
    pub id: i64,
    pub server: Option<String>,
    pub channel: String,
    pub user: String,
    pub command: Option<String>,
    pub stack: Option<String>,
    pub timestamp: Option<i64>,
    pub log: Option<String>,
    pub error: Option<String>,
}
#[derive(Serialize, Deserialize, FromRow)]
pub struct HelpMessage {
    pub id: i64,
    pub cmd: String,
    pub desc: String,
}
#[derive(Serialize, Deserialize, FromRow)]
pub struct Leet {
    pub id: i64,
    pub source: char,
    pub translated: String,
}
#[derive(Serialize, Deserialize, FromRow)]
pub struct AiCommandAlias {
    pub id: i64,
    pub command: String,
    pub alias: String,
}
#[derive(Serialize, Deserialize, FromRow)]
pub struct AiReactions {
    pub id: i64,
    pub command: String,
    pub reaction: String,
}
#[derive(Serialize, Deserialize, FromRow)]
pub struct Prefix {
    pub id: i64,
    #[serde(rename = "serverId")]
    pub server_id: String,
    pub prefix: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct RedditPost {
    pub id: i64,
    pub subreddit: String,
    pub title: String,
    pub url: String,
    pub over_18: bool,
    pub permalink: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StatusType {
    Watching,
    Listening,
    Playing,
    Competing,
    Custom,
}

impl From<String> for StatusType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "watching" => StatusType::Watching,
            "listening" => StatusType::Listening,
            "playing" => StatusType::Playing,
            "competing" => StatusType::Competing,
            _ => StatusType::Custom,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct Status {
    pub id: i64,
    pub r#type: StatusType,
    pub status: String,
}
