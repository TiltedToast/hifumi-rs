use serde::{Deserialize, Serialize};

use crate::utils::inside_docker;
use std::{env, process};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    bot_token: String,
    exchange_api_key: String,
    imgur_client_id: String,
    imgur_client_secret: String,
    reddit_client_id: String,
    reddit_client_secret: String,
    reddit_refresh_token: String,
    mongo_uri: String,
    dev_mode: bool,
}

impl Config {
    pub fn new() -> Self {
        let config = Config {
            bot_token: env::var("BOT_TOKEN").unwrap_or_default(),
            mongo_uri: if inside_docker() {
                "mongodb://db:27017/".to_string()
            } else {
                env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://127.0.0.1:27017/".to_string())
            },
            exchange_api_key: env::var("EXCHANGE_API_KEY").unwrap_or_default(),
            imgur_client_id: env::var("IMGUR_CLIENT_ID").unwrap_or_default(),
            imgur_client_secret: env::var("IMGUR_CLIENT_SECRET").unwrap_or_default(),
            reddit_client_id: env::var("REDDIT_CLIENT_ID").unwrap_or_default(),
            reddit_client_secret: env::var("REDDIT_CLIENT_SECRET").unwrap_or_default(),
            reddit_refresh_token: env::var("REDDIT_REFRESH_TOKEN").unwrap_or_default(),
            dev_mode: env::var("DEV_MODE").unwrap_or_else(|_| "false".to_string()) == "true",
        };

        let missing_credentials = &config.check_config();

        if !missing_credentials.is_empty() {
            println!("Missing credentials: {missing_credentials:?}");
            process::exit(1);
        }

        config
    }

    fn check_config(&self) -> Vec<&str> {
        let mut missing = Vec::new();

        if self.exchange_api_key.is_empty() {
            missing.push("Exchange API Key");
        } else if self.imgur_client_id.is_empty() {
            missing.push("Imgur Client ID");
        } else if self.imgur_client_secret.is_empty() {
            missing.push("Imgur Client Secret");
        } else if self.reddit_client_id.is_empty() {
            missing.push("Reddit Client ID");
        } else if self.reddit_client_secret.is_empty() {
            missing.push("Reddit Client Secret");
        } else if self.reddit_refresh_token.is_empty() {
            missing.push("Reddit Refresh Token");
        }
        missing
    }
}
