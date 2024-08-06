mod commands;
mod config;
mod db;
mod handlers;
mod helpers;

#[macro_use]
extern crate log;

use std::{collections::HashMap, env, io::Write, process};

use anyhow::Result;
use chrono::{format::strftime::StrftimeItems, Utc};
use db::models::{Prefix, Status};
use dotenvy::dotenv;
use log::{Level, LevelFilter};
use pretty_env_logger::{env_logger::fmt::Color, formatted_builder};
use serenity::{async_trait, model::prelude::*, prelude::*, Client as DiscordClient};
use sqlx::SqlitePool;
use tokio::sync::RwLock;

use crate::{
    config::Config,
    handlers::messages::handle_message,
    helpers::{
        types::Handler,
        utils::{error_log, is_indev, start_status_loop},
    },
};

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
                match msg.channel_id.say(&ctx.http, e.to_string()).await {
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
        eprintln!("Failed to load .env file");
        process::exit(1);
    });

    #[rustfmt::skip]
    formatted_builder()
        .filter(Some("hifumi"), LevelFilter::Trace)
        .format(|formatter, record| {
            let mut style = formatter.style();
            match record.level() {
                Level::Trace => style.set_color(Color::Rgb(138, 43, 226)),
                Level::Debug => style.set_color(Color::Rgb(252, 233, 58)),
                Level::Info  => style.set_color(Color::Green),
                Level::Warn  => style.set_color(Color::Rgb(255, 140, 0)),
                Level::Error => style.set_color(Color::Red),
            };
            writeln!(
                formatter,
                "{} {} {}",
                style.value(format_args!(
                    "{}",
                    match record.level() {
                        Level::Info => "INFO ",
                        Level::Warn => "WARN ",
                        _ => record.level().as_str(),
                    }
                )),
                format_args!("{} UTC:", formatter.timestamp()),
                record.args(),
            )
        })
        .init();

    let token = env::var("BOT_TOKEN").unwrap_or_else(|_| {
        error!("Expected a bot token under BOT_TOKEN in the environment");
        process::exit(1);
    });
    let intents = GatewayIntents::all();
    let config = Config::new();

    let db_pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    #[allow(unused_mut)]
    let mut statuses = sqlx::query_as!(Status, "SELECT * FROM statuses")
        .fetch_all(&db_pool)
        .await?;

    let mut prefixes: HashMap<String, String> = HashMap::new();

    let prefix_arr = sqlx::query_as!(Prefix, "SELECT * FROM prefixes")
        .fetch_all(&db_pool)
        .await?;

    for prefix in prefix_arr {
        prefixes.insert(prefix.server_id, prefix.prefix);
    }

    let mut client = DiscordClient::builder(token, intents)
        .event_handler(Handler {
            start_time,
            config,
            db_pool,
            statuses: RwLock::new(statuses),
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
