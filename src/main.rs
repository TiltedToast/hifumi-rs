mod config;
mod utils;

use bson::oid::ObjectId;
use config::Config;
use futures::stream::TryStreamExt;
use mongodb::options::ClientOptions;
use mongodb::Client as MongoClient;
use serde::{Deserialize, Serialize};
use std::{env, process};

use chrono::format::strftime::StrftimeItems;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{async_trait, Client as DiscordClient};

struct Handler {
    pub start_time: DateTime<Utc>,
    pub config: Config,
    pub db_client: MongoClient,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    r#type: String,
    status: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        handle_message(self, ctx, msg).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let date_format = StrftimeItems::new("%d/%m/%Y %H:%M:%S UTC");
        let done_loading_time = Utc::now();
        let done_loading_formatted = done_loading_time.format_with_items(date_format);

        println!(
            "Started up in {}ms on {}",
            done_loading_time.timestamp_millis() - self.start_time.timestamp_millis(),
            done_loading_formatted
        );
        println!("Logged in as:");
        println!("{}", ready.user.name);
        println!("{}", ready.user.id);
        println!("------------------");

        let _ = ctx
            .set_presence(
                Some(Activity::playing("happy new year!")),
                OnlineStatus::Online,
            )
            .await;
    }
}

#[tokio::main]
async fn main() {
    let start_time = Utc::now();
    dotenv().ok();

    let token = env::var("BOT_TOKEN").unwrap_or_else(|_| {
        println!("Expected a bot token under BOT_TOKEN in the environment");
        process::exit(1);
    });
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_EMOJIS_AND_STICKERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let config = Config::new();

    let mongo_options = ClientOptions::parse(&config.mongo_uri)
        .await
        .unwrap_or_else(|_| {
            println!("Failed to parse MongoDB uri");
            process::exit(1);
        });

    let mongo_client = MongoClient::with_options(mongo_options).unwrap_or_else(|_| {
        println!("Failed to connect to MongoDB");
        process::exit(1);
    });

    let status_array = mongo_client
        .database("status")
        .collection::<StatusDoc>("status")
        .find(None, None)
        .await
        .unwrap_or_else(|_| {
            println!("Failed to find status collection");
            process::exit(1);
        })
        .try_collect::<Vec<StatusDoc>>()
        .await
        .unwrap_or_else(|_| {
            println!("Failed to collect status collection");
            process::exit(1);
        });

    for status in status_array {
        println!("{status:?}");
    }

    let mut client = DiscordClient::builder(token, intents)
        .event_handler(Handler {
            start_time,
            config,
            db_client: mongo_client,
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why}");
    }
}

async fn handle_message(_handler: &Handler, ctx: Context, msg: Message) {
    if msg.content == "ping" {
        if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
            println!("Error sending message: {why:?}");
        }
    }
}
