// LastFm API Wrapper

use std::{collections::HashMap, time::Duration};

use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::{
    config::SongConfig,
    utilities::{md5_hash, random_timestamp},
};

pub struct LastFmClient {
    http: reqwest::Client,
    url: String,
    key: String,
    secret: String,
}

impl LastFmClient {
    pub fn new(api_url: &str, app_key: &str, app_secret: &str) -> reqwest::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Length", HeaderValue::from(0));

        let http = reqwest::Client::builder()
            .user_agent("PostmanRuntime/7.37.3")
            .timeout(Duration::from_secs(15))
            .default_headers(headers)
            .build()?;

        Ok(Self {
            http,
            url: api_url.to_string(),
            key: app_key.to_string(),
            secret: app_secret.to_string(),
        })
    }

    pub async fn track_scrobble(
        &self,
        session: &str,
        song: &SongConfig,
    ) -> anyhow::Result<reqwest::Response> {
        let timestamp = random_timestamp()?;
        let signature = md5_hash(&format!(
            "api_key{}artist{}methodtrack.scrobblesk{session}timestamp{timestamp}track{}{}",
            self.key, song.artist, song.title, self.secret
        ));

        let params = ScrobbleParams {
            method: String::from("track.scrobble"),
            artist: song.artist.clone(),
            track: song.title.clone(),
            timestamp,
            api_key: self.key.clone(),
            api_sig: signature,
            sk: session.to_string(),
            format: String::from("json"),
        };

        let res = self.http.post(&self.url).query(&params).send().await?;

        Ok(res)
    }

    pub async fn token(&self) -> reqwest::Result<String> {
        let res: TokenResponse = self
            .http
            .post(&self.url)
            .query(&[
                ("method", "auth.getToken"),
                ("api_key", &self.key),
                ("format", "json"),
            ])
            .send()
            .await?
            .json()
            .await?;

        Ok(res.token)
    }

    pub async fn session(&self, token: &str) -> reqwest::Result<String> {
        let signature = md5_hash(&format!(
            "api_key{}methodauth.getSessiontoken{token}{}",
            self.key, self.secret
        ));

        let res: SessionResponse = self
            .http
            .post(&self.url)
            .query(&[
                ("method", "auth.getSession"),
                ("api_key", &self.key),
                ("token", token),
                ("api_sig", &signature),
                ("format", "json"),
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
