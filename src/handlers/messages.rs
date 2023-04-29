use anyhow::Result;
use serenity::{model::prelude::*, prelude::*};

use crate::{
    commands::misc::user_avatar,
    helpers::{
        types::{Handler, MessageCommandData, PrefixDoc},
        utils::{is_indev, register_prefix},
    },
};

pub async fn handle_message(handler: &Handler<'_>, ctx: &Context, msg: &Message) -> Result<()> {
    if msg.author.bot {
        return Ok(());
    }
    let content = msg
        .content
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<String>>();

    let react_cmd = match content.get(0) {
        Some(cmd) => cmd.strip_prefix('$').unwrap_or_default().to_lowercase(),
        None => String::new(),
    };

    let sub_cmd = match content.get(1) {
        Some(cmd) => cmd.to_lowercase(),
        None => String::new(),
    };

    let prefix_coll = handler
        .db_client
        .database("hifumi")
        .collection::<PrefixDoc>("prefixes");

    if let Some(guild_id) = msg.guild_id {
        if !handler
            .prefixes
            .read()
            .await
            .contains_key(&guild_id.to_string())
        {
            match register_prefix(guild_id, prefix_coll, handler).await {
                Ok(id) => {
                    debug!("Registered prefix for guild: {}", id);
                }
                Err(e) => {
                    error!("Failed to register prefix for guild {}: {}", guild_id, e);
                }
            }
        }
    }

    let prefix = if is_indev() {
        "h?".to_string()
    } else {
        match msg.guild_id {
            Some(id) => handler
                .prefixes
                .read()
                .await
                .get(&id.to_string())
                .map_or_else(|| "h!".to_string(), std::string::ToString::to_string),
            None => "h!".to_string(),
        }
    }
    .to_lowercase();

    if msg.content.to_lowercase().starts_with(&prefix) {
        let command = content[0].replace(&prefix, "");

        debug!("{} used command: {}", msg.author.id, command);

        handle_command(MessageCommandData {
            ctx,
            msg,
            content,
            command,
            react_cmd,
            sub_cmd,
            handler,
            prefix,
        })
        .await?;
    }

    Ok(())
}

async fn handle_command(data: MessageCommandData<'_>) -> Result<()> {
    match data.command.as_str() {
        "pfp" | "avatar" => user_avatar(data).await?,
        _ => {}
    }

    Ok(())
}
