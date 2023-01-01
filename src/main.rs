mod config;
mod utils;

use anyhow::Result as AnyResult;
use bson::oid::ObjectId;
use chrono::format::strftime::StrftimeItems;
use chrono::{DateTime, Utc};
use config::Config;
use dotenv::dotenv;
use futures::stream::TryStreamExt;
use mongodb::options::ClientOptions;
use mongodb::Client as MongoClient;
use serde::{Deserialize, Serialize};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{async_trait, Client as DiscordClient};
use std::collections::HashMap;
use std::{env, process};
use tokio::sync::Mutex;

#[allow(dead_code)]
struct Handler {
    pub start_time: DateTime<Utc>,
    pub config: Config,
    pub db_client: MongoClient,
    pub statuses: Mutex<Vec<StatusDoc>>,
    pub prefixes: Mutex<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StatusDoc {
    _id: ObjectId,
    r#type: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct PrefixDoc {
    _id: ObjectId,
    serverId: String,
    prefix: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(self: &Handler, ctx: Context, msg: Message) {
        match handle_message(self, ctx, msg).await {
            Ok(_) => (),
            Err(e) => println!("Error: {e}"),
        };
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

        let random_status = utils::random_element_vec(&self.statuses.lock().await);

        if let Some(status) = random_status {
            let activity = utils::get_activity((&status.r#type, &status.status));
            ctx.set_activity(activity).await;
        }
    }
}

#[tokio::main]
async fn main() -> AnyResult<()> {
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

    #[allow(unused_mut)]
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
            statuses: Mutex::new(status_array),
            prefixes: Mutex::new(prefixes),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why}");
    }
    Ok(())
}

async fn handle_message(handler: &Handler, ctx: Context, msg: Message) -> AnyResult<()> {
    if msg.author.bot {
        return Ok(());
    }
    let content = msg.content.split_whitespace().collect::<Vec<&str>>();

    let prefix_coll = handler
        .db_client
        .database("hifumi")
        .collection::<PrefixDoc>("prefixes");

    if msg.guild(&ctx).is_some()
        && !handler
            .prefixes
            .lock()
            .await
            .contains_key(&msg.guild_id.unwrap_or_default().to_string())
    {
        let prefix_doc = PrefixDoc {
            _id: ObjectId::new(),
            serverId: match msg.guild_id {
                Some(id) => id.as_u64().to_string(),
                None => return Ok(()),
            },
            prefix: "h!".to_string(),
        };

        prefix_coll.insert_one(&prefix_doc, None).await?;
        handler
            .prefixes
            .lock()
            .await
            .insert(prefix_doc.serverId.to_string(), prefix_doc.prefix);

        msg.channel_id
            .say(
                &ctx.http,
                "I have set the prefix to `h!`. You can change it with `h!prefix`",
            )
            .await?;
    }

    let mut prefix = match msg.guild_id {
        Some(id) => match handler.prefixes.lock().await.get(&id.to_string()) {
            Some(prefix) => prefix.to_string(),
            None => "h!".to_string(),
        },
        None => "h!".to_string(),
    };

    if utils::is_indev() {
        prefix = "h?".to_string();
    }

    if msg
        .content
        .to_lowercase()
        .starts_with(&prefix.to_lowercase())
    {
        let command = content[0].to_lowercase().replace(&prefix, "");

        if command == "ping" {
            msg.channel_id.say(&ctx.http, "Pong!").await?;
        }
    }

    Ok(())
}
