mod utilities;
mod wrapper;
mod config;

use std::time::Duration;

use anyhow::Context;
use reqwest::StatusCode;
use tokio::time;
use rand::seq::SliceRandom;

use crate::{config::load_config, wrapper::LastFmClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().await?;
    let client = LastFmClient::new(&config.api.url, &config.api.key, &config.api.secret)?;

    println!("Getting your user token...");
    let token = client.token().await?;
    println!("A browser will open soon, please authenticate!");

    let auth_url = format!("http://www.last.fm/api/auth/?api_key={}&token={token}", config.api.key);
    open::that(auth_url)?;

    tokio::time::sleep(Duration::from_secs(10)).await;
    // TODO: callback

    let session = client.session(&token).await?;
    println!("Session: {session}");

    let mut interval = time::interval(config.scrobbler.cooldown);
    let mut total_scrobbles = 0;

    loop {
        let song = config
            .scrobbler
            .songs
            .choose(&mut rand::thread_rng())
            .context("You need add at least one song in config file!")?;

        let res = client.track_scrobble(&session, song).await?;

        let status_code = res.status();
        if status_code != StatusCode::OK {
            let body: serde_json::Value = res.json().await?;
            eprintln!("Failed to scrobble song!\n> Status Code: {status_code}\n> Response: {body:?}");
            break;
        }

        total_scrobbles += 1;
        println!("Scrobbled {} - {}! Total scrobbles: {total_scrobbles}", song.artist, song.title);
        interval.tick().await;
    }

    Ok(())
}
