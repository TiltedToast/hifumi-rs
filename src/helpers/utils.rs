use std::env;
use tokio::time::sleep;
use tokio::time::Duration;

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serenity::model::gateway::Activity;
use serenity::prelude::*;

use super::types::StatusVec;

pub async fn start_status_loop(statuses: &StatusVec, ctx: Context) {
    if statuses.lock().await.len() == 0 {
        println!("No statuses found in database");
        return;
    }
    loop {
        let random_status = random_element_vec(&statuses.lock().await);

        if let Some(status) = random_status {
            let activity = get_activity((&status.r#type, &status.status));
            ctx.set_activity(activity).await;
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
pub fn is_indev() -> bool {
    let indev_env = env::var("DEV_MODE").unwrap_or_default();
    indev_env == "true"
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
/// - `STREAMING` -> `Activity::streaming`
/// - `COMPETING` -> `Activity::competing`
pub fn get_activity(status: (&str, &str)) -> Activity {
    match status.0 {
        "LISTENING" => Activity::listening(status.1),
        "WATCHING" => Activity::watching(status.1),
        "STREAMING" => Activity::streaming(status.1, "https://twitch.tv/"),
        "COMPETING" => Activity::competing(status.1),
        _ => Activity::playing(status.1),
    }
}
