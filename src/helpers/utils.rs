use std::env;

use rand::seq::SliceRandom;
use rand::thread_rng;
use serenity::model::gateway::Activity;

pub fn inside_docker() -> bool {
    let docker_env = env::var("DOCKER").unwrap_or_default();
    docker_env == "true"
}

pub fn is_indev() -> bool {
    let indev_env = env::var("DEV_MODE").unwrap_or_default();
    indev_env == "true"
}

pub fn random_element_vec<T: Clone>(vec: &[T]) -> Option<T> {
    let mut rng = thread_rng();
    vec.choose(&mut rng).cloned()
}

pub fn get_activity(status: (&str, &str)) -> Activity {
    match status.0 {
        "LISTENING" => Activity::listening(status.1),
        "WATCHING" => Activity::watching(status.1),
        "STREAMING" => Activity::streaming(status.1, "https://twitch.tv/"),
        "COMPETING" => Activity::competing(status.1),
        _ => Activity::playing(status.1),
    }
}
