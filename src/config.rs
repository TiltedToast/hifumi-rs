use serenity::utils::{Color, Colour};

use crate::helpers::utils::inside_docker;
use std::{env, process};

#[derive(Debug)]
pub struct Config<'a> {
    pub bot_token: String,
    pub exchange_api_key: String,
    pub imgur_client_id: String,
    pub imgur_client_secret: String,
    pub reddit_client_id: String,
    pub reddit_client_secret: String,
    pub reddit_refresh_token: String,
    pub mongo_uri: String,
    pub dev_mode: bool,
    pub embed_colour: Color,
    pub dev_channels: &'a [u64],
    pub bot_owners: &'a [u64],
    pub log_channel: u64,
}

impl Config<'_> {
    pub fn new() -> Self {
        let config = Config {
            bot_token: env::var("BOT_TOKEN").unwrap_or_default(),
            mongo_uri: if inside_docker() {
                "mongodb://db:27017/".to_string()
            } else {
                env::var("MONGO_URI").unwrap_or("mongodb://127.0.0.1:27017/".to_string())
            },
            exchange_api_key: env::var("EXCHANGE_API_KEY").unwrap_or_default(),
            imgur_client_id: env::var("IMGUR_CLIENT_ID").unwrap_or_default(),
            imgur_client_secret: env::var("IMGUR_CLIENT_SECRET").unwrap_or_default(),
            reddit_client_id: env::var("REDDIT_CLIENT_ID").unwrap_or_default(),
            reddit_client_secret: env::var("REDDIT_CLIENT_SECRET").unwrap_or_default(),
            reddit_refresh_token: env::var("REDDIT_REFRESH_TOKEN").unwrap_or_default(),
            dev_mode: env::var("DEV_MODE").unwrap_or_default() == "true",
            embed_colour: Colour::from(0xCE_3A_9B),
            dev_channels: &[655484859405303809, 551588329003548683, 922679249058553857],
            bot_owners: &[258993932262834188, 207505077013839883],
            log_channel: 655484804405657642,
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
