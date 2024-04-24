use std::time::Duration;

use serde::Deserialize;
use serde_with::serde_as;
use tokio::fs;

const CONFIG_FILE: &str = "Config.toml";

pub async fn load_config() -> anyhow::Result<Config> {
    let contents = fs::read_to_string(CONFIG_FILE).await?;
    let parsed = toml::from_str(&contents)?;
    Ok(parsed)
}

#[derive(Deserialize)]
pub struct ApiConfig {
    pub url: String,
    pub key: String,
    pub secret: String,
}

#[derive(Deserialize)]
pub struct SongConfig {
    pub artist: String,
    pub title: String,
}

#[serde_as]
#[derive(Deserialize)]
pub struct ScrobblerConfig {
    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    pub cooldown: Duration,
    pub songs: Vec<SongConfig>,
}

#[derive(Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub scrobbler: ScrobblerConfig,
}
