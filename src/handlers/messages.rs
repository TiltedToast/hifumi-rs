use bson::oid::ObjectId;
use serenity::prelude::*;
use serenity::model::prelude::*;
use anyhow::Result as AnyResult;

use crate::helpers::{utils::is_indev, types::{PrefixDoc, Handler}};

pub async fn handle_message(handler: &Handler, ctx: Context, msg: Message) -> AnyResult<()> {
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

    if is_indev() {
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