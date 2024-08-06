use anyhow::{anyhow, Result};
use serenity::{all::CreateMessage, builder::CreateEmbed};

use crate::helpers::{types::MessageCommandData, utils::parse_target_user};

pub async fn user_avatar(data: MessageCommandData<'_>) -> Result<()> {
    let user = parse_target_user(&data, 1).await?;

    let embed = CreateEmbed::default()
        .title(format!("{}'s avatar", user.name))
        .image(user.face())
        .color(data.handler.config.embed_colour);

    data.msg
        .channel_id
        .send_message(&data.ctx, CreateMessage::default().add_embed(embed))
        .await
        .map_err(|_| anyhow!("Failed to send message"))?;

    Ok(())
}
