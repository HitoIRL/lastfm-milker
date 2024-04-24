use std::{collections::HashMap, time::{Duration, SystemTime}};

use dotenvy_macro::dotenv;
use md5::{Md5, Digest};
use rand::Rng;
use reqwest::{header::{HeaderMap, HeaderValue}, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::time;

const API_URL: &str = dotenv!("API_URL");
const API_KEY: &str = dotenv!("API_KEY");
const SECRET: &str = dotenv!("SECRET");

// wrapper around lastfm api
struct LastFmClient {
    http: reqwest::Client,
}

impl LastFmClient {
    pub fn new() -> reqwest::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Length", HeaderValue::from(0));

        let http = reqwest::Client::builder()
            .user_agent("PostmanRuntime/7.37.3")
            .timeout(Duration::from_secs(15))
            .default_headers(headers)
            .build()?;

        Ok(Self { http })
    }

    pub async fn track_scrobble(&self, session: &str) -> reqwest::Result<reqwest::Response> {
        let artist = "$uicideboy$";
        let track = "LTE";
        let timestamp = random_timestamp();

        let signature = md5_hash(&format!("api_key{API_KEY}artist{artist}methodtrack.scrobblesk{session}timestamp{timestamp}track{track}{SECRET}"));

        let params = ScrobbleParams {
            method: String::from("track.scrobble"),
            artist: artist.to_string(),
            track: track.to_string(),
            timestamp,
            api_key: API_KEY.to_string(),
            api_sig: signature,
            sk: session.to_string(),
            format: String::from("json"),
        };

        let res = self
            .http
            .post(API_URL)
            .query(&params)
            .send()
            .await?;

        Ok(res)
    }

    pub async fn token(&self) -> reqwest::Result<String> {
        let res: TokenResponse = self
            .http
            .post(API_URL)
            .query(&[
                ("method", "auth.getToken"),
                ("api_key", API_KEY),
                ("format", "json")
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(res.token)
    }

    pub async fn session(&self, token: &str) -> reqwest::Result<String> {
        let signature = md5_hash(&format!("api_key{API_KEY}methodauth.getSessiontoken{token}{SECRET}"));

        let res: SessionResponse = self
            .http
            .post(API_URL)
            .query(&[
                ("method", "auth.getSession"),
                ("api_key", API_KEY),
                ("token", token),
                ("api_sig", &signature),
                ("format", "json")
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(res.session.key)
    }
}

#[derive(Serialize)]
struct ScrobbleParams {
    method: String,
    artist: String,
    track: String,
    timestamp: u64,
    api_key: String,
    api_sig: String,
    sk: String,
    format: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Deserialize)]
struct Session {
    key: String,
    #[serde(flatten)]
    _extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct SessionResponse {
    session: Session,
}

fn md5_hash(plaintext: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(plaintext.as_bytes());
    let result = hasher.finalize();

    hex::encode(result)
}

/// generates random timestamp within this week
fn random_timestamp() -> u64 {
    let current_date = SystemTime::now();

    let start_of_week = {
        let days_since_sunday = current_date.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() % (7 * 24 * 3600);
        current_date - Duration::from_secs(days_since_sunday)
    };

    let end_of_week = start_of_week + Duration::from_secs(6 * 24 * 3600);

    let mut rng = rand::thread_rng();
    let mut random_timestamp;

    loop {
        let random_duration = rng.gen_range(0..=(end_of_week.duration_since(start_of_week).unwrap().as_secs()));
        random_timestamp = start_of_week + Duration::from_secs(random_duration);

        if random_timestamp <= current_date {
            break;
        }
    }

    random_timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = LastFmClient::new()?;
    println!("Getting your user token...");
    let token = client.token().await?;
    println!("A browser will open soon, please authenticate!");

    let auth_url = format!("http://www.last.fm/api/auth/?api_key={API_KEY}&token={token}");
    open::that(auth_url)?;

    tokio::time::sleep(Duration::from_secs(10)).await;
    // TODO: callback

    let session = client.session(&token).await?;
    println!("Session: {session}");

    let mut interval = time::interval(Duration::from_millis(1500)); // i had problems when settings this below 1.5 seconds
    let mut total_scrobbles = 0;

    loop {
        let res = client.track_scrobble(&session).await?;

        let status_code = res.status();
        if status_code != StatusCode::OK {
            eprintln!("Failed to scrobble song, status code: {status_code}, response: {res:?}");
            break;
        }

        total_scrobbles += 1;
        println!("Scrobbled song! Total scrobbles: {total_scrobbles}");
        interval.tick().await;
    }

    Ok(())
}
