use std::env;

pub fn inside_docker() -> bool {
    let docker_env = env::var("DOCKER").unwrap_or_default();
    docker_env == "true"
}
