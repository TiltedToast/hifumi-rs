#![allow(clippy::unreadable_literal)]

use std::env;

use anyhow::{anyhow, Result};
use bson::oid::ObjectId;
use chrono::{format::strftime::StrftimeItems, Utc};
use mongodb::Collection;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serenity::{
    model::{
        gateway::Activity,
        prelude::{ChannelId, GuildId, Message},
        user::User,
    },
    prelude::*,
};
use tokio::time::{sleep, Duration};

use super::types::{Handler, MessageCommandData, PrefixDoc, StatusVec};

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
    let current_time = Utc::now().format_with_items(date_format);

    let error_channel = message
        .channel_id
        .name(&ctx)
        .await
        .unwrap_or("Unknown".into());

    let guild_name = match message.guild(ctx) {
        Some(guild) => guild.name,
        None => "Direct Message".to_string(),
    };

    let guild_id = match message.guild_id {
        Some(id) => id.to_string(),
        None => "Unknown".to_string(),
    };

    let (user_name, user_id) = (&message.author.name, message.author.id);

    let error_msg = String::new()
        + &format!("An Error occurred on {current_time}\n")
        + &format!("**Server:** {guild_name} - {guild_id}\n")
        + &format!("**Room:** {error_channel}\n")
        + &format!("**User:** {user_name} - {user_id}\n",)
        + &format!("**Command used:** {}\n", message.content)
        + &format!("**Error:** {error}");

    error!("An Error occurred on {current_time}");
    error!("Server: {guild_name} - {guild_id}");
    error!("Room: {error_channel}");
    error!("User: {user_name} - {user_id}");
    error!("Command used: {}", message.content);
    error!("Error: {error}");

    let error_channel = if handler
        .config
        .dev_channels
        .contains(message.channel_id.as_u64())
    {
        message.channel_id
    } else {
        ChannelId(handler.config.log_channel)
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
            .get_user(user_id)
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
/// * `prefix_coll` - The `MongoDB` collection for the prefixes
/// * `handler` - The Event Handler that dispatches the events
///
/// # Returns
///
/// * `String` - If the prefix was successfully registered, returns the guild Id
/// * `Err` - If the prefix was not registered
///
/// # Errors
/// * If inserting the prefix into the database fails
pub async fn register_prefix(
    guild_id: GuildId,
    prefix_coll: Collection<PrefixDoc>,
    handler: &Handler<'_>,
) -> Result<String> {
    let prefix_doc = PrefixDoc {
        _id:       ObjectId::new(),
        server_id: guild_id.to_string(),
        prefix:    String::from("h!"),
    };
    prefix_coll.insert_one(&prefix_doc, None).await?;

    handler
        .prefixes
        .write()
        .await
        .insert(prefix_doc.server_id.clone(), prefix_doc.prefix);

    Ok(prefix_doc.server_id)
}

/// A function that takes a vector of statuses and a context
/// and sets the bot's status to a random status from the vector every 5-15
/// minutes.
pub async fn start_status_loop(statuses: &StatusVec, ctx: Context) {
    loop {
        let random_status = random_element_vec(&statuses.read().await);

        if let Some(status_doc) = random_status {
            let activity = get_activity(&status_doc.r#type, &status_doc.status);
            ctx.set_activity(activity).await;
            debug!("Set status to: {} {}", status_doc.r#type, status_doc.status);
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

/// Check if the bot is running inside a docker container
///
/// Checks for `DOCKER` environment variable to be set to `anything` as part
/// of the Dockerfile
pub fn inside_docker() -> bool {
    !env::var("DOCKER").unwrap_or_default().is_empty()
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
/// Maps a string and text to a serenity activity
///
/// The first string is the type of activity, the second is the text to use for the activity
///
/// The possible values are:
/// - `WATCHING` -> `Activity::watching`
/// - `LISTENING` -> `Activity::listening`
/// - `PLAYING` -> `Activity::playing`
/// - `COMPETING` -> `Activity::competing`
///
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
/// let activity = get_activity("WATCHING", "Star Wars");
/// assert_eq!(activity, Activity::watching("Star Wars"));
///
/// let activity = get_activity("EATING", "Pizza");
/// assert_eq!(activity, Activity::playing("Pizza")
/// ```
pub fn get_activity(r#type: &str, status_msg: &str) -> Activity {
    match r#type.to_lowercase().as_str() {
        "listening" => Activity::listening(status_msg),
        "watching"  => Activity::watching(status_msg),
        "competing" => Activity::competing(status_msg),
        _ => Activity::playing(status_msg),
    }
}
