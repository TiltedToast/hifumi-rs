use anyhow::{anyhow, Result};
use serenity::builder::CreateEmbed;

use crate::helpers::{types::MessageCommandData, utils::parse_target_user};

pub async fn user_avatar(data: MessageCommandData<'_>) -> Result<()> {
    debug!("{} used 'user_avatar' command", data.msg.author.id);
    let user = parse_target_user(&data, 1).await?;

    let embed = CreateEmbed::default()
        .title(format!("{}'s avatar", user.name))
        .image(user.face())
        .color(data.handler.config.embed_colour)
        .clone();

    data.msg
        .channel_id
        .send_message(&data.ctx, |m| m.set_embed(embed))
        .await
        .map_err(|_| anyhow!("Failed to send message"))?;

    Ok(())
}
