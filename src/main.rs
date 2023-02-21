mod commands;
mod config;
mod handlers;
mod helpers;

use crate::config::Config;
use crate::handlers::messages::handle_message;
use crate::helpers::types::{Handler, PrefixDoc, StatusDoc};
use crate::helpers::utils::{is_indev, start_status_loop};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use anyhow::Result;
use chrono::format::strftime::StrftimeItems;
use chrono::Utc;
use dotenvy::dotenv;
use futures::stream::TryStreamExt;
use helpers::utils::error_log;
use mongodb::options::ClientOptions;
use mongodb::Client as MongoClient;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{async_trait, Client as DiscordClient};
use std::collections::HashMap;
use std::{env, process};
use tokio::sync::RwLock;

#[async_trait]
impl EventHandler for Handler<'_> {
    async fn message(&self, ctx: Context, msg: Message) {
        match handle_message(self, &ctx, &msg).await {
            Ok(_) => (),
            Err(e) => {
                match error_log(&msg, &e, &ctx, self).await {
                    Ok(_) => (),
                    Err(e) => error!("Failed to log error, {e}"),
                }
                match msg.channel_id.say(&ctx.http, e).await {
                    Ok(_) => (),
                    Err(e) => error!("Failed to send message, {e}"),
                };
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let date_format = StrftimeItems::new("%d/%m/%Y %H:%M:%S UTC");
        let done_loading_time = Utc::now();
        let done_loading_formatted = done_loading_time.format_with_items(date_format);

        info!(
            "Started up in {}ms on {}",
            done_loading_time.timestamp_millis() - self.start_time.timestamp_millis(),
            done_loading_formatted
        );
        info!("Logged in as:");
        info!("{}", ready.user.name);
        info!("{}", ready.user.id);
        info!("------------------");

        let status_loop = start_status_loop(&self.statuses, ctx);

        if is_indev() {
            info!("Running in dev mode");
        } else {
            info!("Running in production mode");
        }

        futures::join!(status_loop);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Utc::now();
    dotenv().unwrap_or_else(|_| {
        error!("Failed to load .env file");
        process::exit(1);
    });
    pretty_env_logger::init();

    let token = env::var("BOT_TOKEN").unwrap_or_else(|_| {
        error!("Expected a bot token under BOT_TOKEN in the environment");
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
            error!("Failed to parse MongoDB URI");
            process::exit(1);
        });

    let mongo_client = MongoClient::with_options(mongo_options).unwrap_or_else(|_| {
        error!("Failed to connect to MongoDB");
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

    for prefix_doc in prefix_array {
        prefixes.insert(prefix_doc.serverId, prefix_doc.prefix);
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
        .unwrap_or_else(|err| {
            error!("Error creating client: {err:?}");
            process::exit(1);
        });

    if let Err(why) = client.start().await {
        error!("Client error: {why}");
    }
    Ok(())
}
