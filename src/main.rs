mod config;
mod handlers;
mod helpers;

use crate::config::Config;
use crate::handlers::messages::handle_message;
use crate::helpers::types::{Handler, PrefixDoc, StatusDoc};
use crate::helpers::utils::{is_indev, start_status_loop};

use anyhow::Result;
use chrono::format::strftime::StrftimeItems;
use chrono::Utc;
use dotenv::dotenv;
use futures::stream::TryStreamExt;
use mongodb::options::ClientOptions;
use mongodb::Client as MongoClient;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{async_trait, Client as DiscordClient};
use std::collections::HashMap;
use std::{env, process};
use tokio::sync::RwLock;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match handle_message(self, &ctx, &msg).await {
            Ok(_) => (),
            Err(e) => {
                msg.channel_id.say(&ctx.http, e).await.unwrap();
            }
        }
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

        let status_loop = start_status_loop(&self.statuses, ctx);

        if is_indev() {
            status_loop.await;
        } else {
            println!("Running in production mode");
            status_loop.await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
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

    #[allow(unused_mut)]
    let mut status_array = mongo_client
        .database("hifumi")
        .collection::<StatusDoc>("statuses")
        .find(None, None)
        .await?
        .try_collect::<Vec<StatusDoc>>()
        .await?;

    let mut prefixes: HashMap<String, String> = HashMap::new();

    let prefix_array = mongo_client
        .database("hifumi")
        .collection::<PrefixDoc>("prefixes")
        .find(None, None)
        .await?
        .try_collect::<Vec<PrefixDoc>>()
        .await?;

    for prefix in prefix_array {
        prefixes.insert(prefix.serverId.to_string(), prefix.prefix.to_string());
    }

    let mut client = DiscordClient::builder(token, intents)
        .event_handler(Handler {
            start_time,
            config,
            db_client: mongo_client,
            statuses: RwLock::new(status_array),
            prefixes: RwLock::new(prefixes),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why}");
    }
    Ok(())
}
