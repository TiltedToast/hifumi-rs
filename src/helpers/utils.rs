#![allow(clippy::unreadable_literal)]

use std::env;

use anyhow::{anyhow, Result};
use chrono::{format::strftime::StrftimeItems, Utc};
use rand::{seq::SliceRandom, thread_rng, Rng};
use serenity::{
    all::{ActivityData, UserId},
    model::{
        prelude::{ChannelId, GuildId, Message},
        user::User,
    },
    prelude::*,
};
use tokio::time::{sleep, Duration};

use super::types::{Handler, MessageCommandData, StatusVec};
use crate::db::models::StatusType;

/// Logs an error to the console and to the error channel.
/// Also saves it to the database.
///
/// # Arguments
/// * `message` - The message that caused the error.
/// * `error` - The error that occurred.
/// * `ctx` - The context of the message.
/// * `handler` - The event handler of the bot.
///
/// TODO: Add database logging.
pub async fn error_log(
    message: &Message,
    error: &anyhow::Error,
    ctx: &Context,
    handler: &Handler<'_>,
) -> Result<()> {
    let date_format = StrftimeItems::new("%d/%m/%Y %H:%M:%S UTC");
    let current_time = Utc::now().format_with_items(date_format).to_string();

    let error_channel = message
        .channel_id
        .name(&ctx)
        .await
        .unwrap_or("Unknown".into());

    let guild_name = match message.guild(&ctx.cache) {
        Some(guild) => guild.name.clone(),
        None => "Direct Message".to_string(),
    };

    let guild_id = match message.guild_id {
        Some(id) => id.to_string(),
        None => "Unknown".to_string(),
    };

    let (user_name, user_id) = (&message.author.name, message.author.id);

    let error_msg = String::new()
        + &format!("An Error occurred on {}\n", &current_time)
        + &format!("**Server:** {} - {}\n", &guild_name, &guild_id)
        + &format!("**Room:** {}\n", &error_channel)
        + &format!("**User:** {} - {}\n", &user_name, &user_id)
        + &format!("**Command used:** {}\n", message.content)
        + &format!("**Error:** {}", &error);

    error!("An Error occurred on {}", &current_time);
    error!("Server: {} - {}", &guild_name, &guild_id);
    error!("Room: {}", &error_channel);
    error!("User: {} - {}", &user_name, &user_id);
    error!("Command used: {}", message.content);
    error!("Error: {}", &error);

    let error_channel = if handler
        .config
        .dev_channels
        .contains(&message.channel_id.into())
    {
        message.channel_id
    } else {
        ChannelId::new(handler.config.log_channel)
    };

    error_channel.say(&ctx.http, &error_msg).await?;

    Ok(())
}

/// Parses a user from the message content at the given index.
/// If no user is found, the author of the message is returned.
/// If the user is not found, an error is returned.
///
/// # Arguments
/// * `data` - The message command data.
/// * `idx` - The index of the user in the message content.
///
/// # Errors
/// * If the user is not found.
/// * If the user ID is not a valid u64.
///
/// # Returns
/// The target user.
pub async fn parse_target_user<'a>(data: &MessageCommandData<'a>, idx: usize) -> Result<User> {
    let user = if data.content.get(idx).is_some() {
        let user_id = data.content[idx].replace("<@", "").replace('>', "");
        let user_id = user_id
            .parse::<u64>()
            .map_err(|_| anyhow!("Invalid User Id"))?;
        data.ctx
            .http
            .get_user(UserId::from(user_id))
            .await
            .map_err(|_| anyhow!("User not found"))?
    } else {
        data.msg.author.clone()
    };
    Ok(user)
}

/// Registers the prefix for the guild in the database and in the prefixes map
///
/// # Arguments
///
/// * `guild_id` - The Id of the guild to register the prefix for
/// * `handler` - The Event Handler that dispatches the events
///
/// # Returns
///
/// * `String` - If the prefix was successfully registered, returns the guild Id
/// * `Err` - If the prefix was not registered
///
/// # Errors
/// * If inserting the prefix into the database fails
pub async fn register_prefix(guild_id: GuildId, handler: &Handler<'_>) -> Result<String> {
    let server_id = guild_id.to_string();
    let prefix = String::from("h!");

    sqlx::query!(
        "INSERT INTO prefixes (server_id, prefiX) VALUES (?, ?)",
        server_id,
        prefix,
    )
    .execute(&handler.db_pool)
    .await?;

    handler
        .prefixes
        .write()
        .await
        .insert(server_id.clone(), prefix);

    Ok(server_id)
}

/// A function that takes a vector of statuses and a context
/// and sets the bot's status to a random status from the vector every 5-15
/// minutes.
pub async fn start_status_loop(statuses: &StatusVec, ctx: Context) {
    loop {
        let random_status = random_element_vec(&statuses.read().await);

        if let Some(status) = random_status {
            let activity = get_activity(&status.r#type, &status.status);
            ctx.set_activity(Some(activity));
            debug!("Set status to: {:?} {}", status.r#type, status.status);
        } else {
            error!("No statuses found in database");
            return;
        }
        sleep(Duration::from_secs(random_int_from_range(300, 900))).await; // 5-15 minutes
    }
}

/// Generate a random number between the given bounds
///
/// # Arguments
/// * `min` - The minimum number (inclusive)
/// * `max` - The maximum number (inclusive)
///
/// # Example
/// ```
/// use helpers::utils;
/// let random_number = utils::random_int_from_range(1, 10);
/// assert!(random_number >= 1 && random_number <= 10);
pub fn random_int_from_range(min: u64, max: u64) -> u64 {
    thread_rng().gen_range(min..=max)
}


/// Checks if the current environment is in development mode.
///
/// Checks for `DEV_MODE` environment variable to be set to `true`
pub fn is_indev() -> bool {
    env::var("DEV_MODE").unwrap_or_default() == "true"
}

/// Returns a random item from a slice, Some(item) if the slice is not empty,
/// None otherwise.
///
/// # Examples
///
/// ```
/// use helpers::utils;
/// let slice = [1, 2, 3, 4, 5];
/// let random = utils::random_item(&slice);
/// assert!(random.is_some());
///
/// let slice = [];
/// let random = utils::random_item(&slice);
/// assert!(random.is_none());
/// ```
pub fn random_element_vec<T: Clone>(vec: &[T]) -> Option<T> {
    let mut rng = thread_rng();
    vec.choose(&mut rng).cloned()
}

#[rustfmt::skip]
/// Returns a Discord activity based on the status type and name.
///
/// # Arguments
///
/// * `r#type` - The status type.
/// * `status_msg` - The status message
///
/// # Examples
///
/// ```
/// let activity = get_activity(StatusType::Watching, "Star Wars");
/// assert_eq!(activity, Activity::watching("Star Wars"));
///
/// let activity = get_activity(StatusType::Playing, "with Rust");
/// assert_eq!(activity, Activity::playing("with Rust")
/// ```
pub fn get_activity(r#type: &StatusType, status_msg: &str) -> ActivityData {
    match r#type {
        StatusType::Listening => ActivityData::listening(status_msg),
        StatusType::Watching  => ActivityData::watching(status_msg),
        StatusType::Competing => ActivityData::competing(status_msg),
        StatusType::Custom    => ActivityData::custom(status_msg),
        StatusType::Playing   => ActivityData::playing(status_msg),
    }
}
