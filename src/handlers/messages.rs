use anyhow::Result as AnyResult;
use bson::oid::ObjectId;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::helpers::{
    types::{Handler, MessageCommandData, PrefixDoc},
    utils::is_indev,
};

pub async fn handle_message(handler: &Handler, ctx: Context, msg: Message) -> AnyResult<()> {
    if msg.author.bot {
        return Ok(());
    }
    let content = msg
        .content
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<String>>();

    println!("{content:?}");

    if content.is_empty() {
        return Ok(());
    }

    let react_cmd = content[0].strip_prefix('$').unwrap_or_default().to_string();

    let mut sub_cmd = String::new();

    if content.len() > 1 {
        sub_cmd = content[1].to_string();
    }

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

    prefix = prefix.to_lowercase();

    if msg.content.starts_with(&prefix) {
        let command = content[0].replace(&prefix, "");

        handle_command(MessageCommandData {
            ctx,
            msg,
            command,
            react_cmd,
            sub_cmd,
        })
        .await?;
    }

    Ok(())
}

async fn handle_command(data: MessageCommandData) -> AnyResult<()> {
    if data.command == "ping" {
        data.msg.channel_id.say(&data.ctx.http, "Pong!").await?;
    }

    Ok(())
}
