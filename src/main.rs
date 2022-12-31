use std::env;

use chrono::format::strftime::StrftimeItems;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{async_trait, Client};

struct Handler {
    start_time: DateTime<Utc>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        handle_message(ctx, msg).await;
    }

    async fn ready(&self, _: Context, ready: Ready) {
        let date_format = StrftimeItems::new("%d/%m/%Y %H:%M:%S");
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
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_EMOJIS_AND_STICKERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler {
            start_time: Utc::now(),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why}");
    }
}

async fn handle_message(ctx: Context, msg: Message) {
    if msg.content == "ping" {
        if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
            println!("Error sending message: {why:?}");
        }
    }
}
