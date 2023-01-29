#![allow(clippy::unreadable_literal)]

use anyhow::anyhow;
use anyhow::Result;
use bson::oid::ObjectId;
use chrono::format::strftime::StrftimeItems;
use chrono::Utc;
use mongodb::Collection;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::Message;
use serenity::model::user::User;
use std::env;
use tokio::time::sleep;
use tokio::time::Duration;

use super::types::Handler;
use super::types::MessageCommandData;
use super::types::PrefixDoc;
use super::types::StatusVec;

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serenity::model::gateway::Activity;
use serenity::prelude::*;

/// Logs an error to the console and to the error channel.
/// Also saves it to the database.
///
/// # Arguments
/// * `message` - The message that caused the error.
/// * `error` - The error that occurred.
/// * `ctx` - The context of the message.
/// * `handler` - The event handler of the bot.
///
/// TODO: Figure out how to actually allow passing in an error directly.
pub async fn error_log(
    message: &Message,
    error: String,
    ctx: &Context,
    handler: &Handler<'_>,
) -> Result<()> {
    let date_format = StrftimeItems::new("%d/%m/%Y %H:%M:%S UTC");
    let current_time = Utc::now().format_with_items(date_format);

    let error_channel = message
        .channel_id
        .name(&ctx)
        .await
        .unwrap_or_else(|| "Unknown channel".into());

    let guild_name = match message.guild(ctx) {
        Some(guild) => guild.name,
        None => "Direct Message".to_string(),
    };

    let guild_id = message
        .guild_id
        .map_or_else(|| "Unknown".to_string(), |id| id.to_string());

    let (user_name, user_id) = (&message.author.name, message.author.id);

    let error_msg = String::new()
        + &format!("An Error occurred on {current_time}\n")
        + &format!("**Server:** {guild_name} - {guild_id}\n")
        + &format!("**Room:** {error_channel}\n")
        + &format!("**User:** {user_name} - {user_id}\n",)
        + &format!("**Command used:** {}\n", message.content)
        + &format!("**Error:** {error}");

    let error_channel = if handler
        .config
        .dev_channels
        .contains(message.channel_id.as_u64())
    {
        message.channel_id
    } else {
        ChannelId(handler.config.log_channel)
    };

    error_channel
        .say(&ctx.http, &error_msg)
        .await
        .map_err(|_| anyhow!("Error sending error message"))?;

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
/// * `msg` - The message that triggered the command
/// * `prefix_coll` - The `MongoDB` collection for the prefixes
/// * `handler` - The Event Handler that dispatches the events
///
/// # Returns
///
/// * `()` - If the prefix was successfully registered
/// * `Err` - If the prefix was not registered
///
/// # Errors
/// * If the guild id is not found even though the message is from a guild (should never happen)
/// * If inserting the prefix into the database fails
pub async fn register_prefix(
    msg: &Message,
    prefix_coll: Collection<PrefixDoc>,
    handler: &Handler<'_>,
) -> Result<()> {
    let prefix_doc = PrefixDoc {
        _id: ObjectId::new(),
        serverId: match msg.guild_id {
            Some(id) => id.to_string(),
            None => return Err(anyhow!("No guild id found")),
        },
        prefix: "h!".to_string(),
    };
    prefix_coll.insert_one(&prefix_doc, None).await?;
    handler
        .prefixes
        .write()
        .await
        .insert(prefix_doc.serverId, prefix_doc.prefix);

    Ok(())
}

/// A function that takes a vector of statuses and a context
/// and sets the bot's status to a random status from the vector every 5-15 minutes.
pub async fn start_status_loop(statuses: &StatusVec, ctx: Context) {
    loop {
        let random_status = random_element_vec(&statuses.read().await);

        if let Some(status) = random_status {
            let activity = get_activity((&status.r#type, &status.status));
            ctx.set_activity(activity).await;
        } else {
            println!("No statuses found in database");
            return;
        }
        sleep(Duration::from_secs(random_int_from_range(300, 900))).await;
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
/// use helpers::utils::random_int_from_range;
/// let random_number = random_int_from_range(1, 10);
/// assert!(random_number >= 1 && random_number <= 10);
pub fn random_int_from_range(min: u64, max: u64) -> u64 {
    let mut rng = thread_rng();
    Rng::gen_range(&mut rng, min..=max)
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

/// Returns a random item from a slice, Some(item) if the slice is not empty, None otherwise.
///
/// # Examples
///
/// ```
/// use helpers::utils;
/// let slice = [1, 2, 3, 4, 5];
/// let random = utils::random_item(&slice);
/// ```
pub fn random_element_vec<T: Clone>(vec: &[T]) -> Option<T> {
    let mut rng = thread_rng();
    vec.choose(&mut rng).cloned()
}

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
/// * `status` - A tuple containing the status type and name.
///
/// # Examples
///
/// ```
/// let status = ("WATCHING", "Star Wars");
/// let activity = get_activity(status);
///
/// assert_eq!(activity, Activity::watching("Star Wars"));
/// ```
pub fn get_activity(status: (&str, &str)) -> Activity {
    match status.0.to_lowercase().as_str() {
        "listening" => Activity::listening(status.1),
        "watching" => Activity::watching(status.1),
        "competing" => Activity::competing(status.1),
        _ => Activity::playing(status.1),
    }
}
