pub mod html;
pub mod llm;
pub mod metadata;
pub mod parser;
pub mod verify;

use crate::config::{ENV, state::AppState};
use crate::pages::status::{ratelimit, servererror};
use askama::Template;
use axum::Json;
use axum::http::header;
use axum::http::{HeaderName, StatusCode};
use axum::response::{Html, IntoResponse};
use base64::prelude::*;
use chrono::{DateTime, FixedOffset, Utc};
use governor::middleware::NoOpMiddleware;
use phf::phf_map;
use reqwest::Client;
use serde::{Deserialize, Deserializer};
use serde_json::json;
use std::collections::HashMap;
use std::{fmt::Display, sync::Arc};
use tokio::signal;
use tower_governor::governor::GovernorConfig;
use tower_governor::{
    GovernorError, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
};
use ulid::Ulid;

pub static SOCIALS: phf::Map<&'static str, &'static str> = phf_map! {
    "github" => "https://github.com/VinukaThejana",
    "git" => "https://github.com/VinukaThejana",
    "substack" => "https://vinuka.substack.com/",
    "linkedin" => "https://www.linkedin.com/in/vinukakodituwakku/",
    "in" => "https://www.linkedin.com/in/vinukakodituwakku/",
    "twitter" => "https://twitter.com/VinukaThejana",
    "x" => "https://twitter.com/VinukaThejana",
    "instagram" => "https://www.instagram.com/vinukathejana/",
    "ig" => "https://www.instagram.com/vinukathejana/",
    "facebook" => "https://www.facebook.com/vinukakodituwakku",
    "fb" => "https://www.facebook.com/vinukakodituwakku",
    "threads" => "https://www.threads.com/@vinukathejana",
};

pub const AUTHOR: &str = "Vinuka Kodituwakku";
pub const AUTHOR_EMAIL: &str = "vinuka.t@icloud.com";
pub const AUTHOR_GITHUB: &str = "https://github.com/VinukaThejana";
pub const AUTHOR_TWITTER: &str = "@VinukaThejana";

pub const POST_LIMIT: usize = 10;

pub const NON_EXISTENT_KEY: &str = "non-existent-post";

pub const IMG_EXTENSIONS: [&str; 7] = [".png", ".jpg", ".jpeg", ".gif", ".bmp", ".webp", ".svg"];

pub async fn shutdown(state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c().await.unwrap_or_else(|_| {
            log::error!("failed to listen for the ctrl+c signal");
            std::process::exit(1);
        })
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap_or_else(|_| {
                log::error!("failed to listen for the SIGTERM signal");
                std::process::exit(1);
            })
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            log::info!("recieved ctrl+c signal");
        }
        _ = terminate => {
            log::info!("received SIGTERM signal");
        }
    };

    log::info!("shutting down ... ");
    state.close().await;
}

pub fn deserialize_base64<'de, D>(deserializer: D) -> Result<Arc<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    let bytes = BASE64_STANDARD
        .decode(s.as_bytes())
        .map_err(serde::de::Error::custom)?;

    Ok(Arc::new(bytes))
}

pub fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Ok(s.into())
}

pub fn to_cache<T>(item: &T) -> String
where
    T: serde::Serialize,
{
    serde_json::to_string(item).unwrap_or_else(|_| String::from("[]"))
}

pub fn from_cache<T>(payload: &str) -> T
where
    T: serde::de::DeserializeOwned + Default,
{
    serde_json::from_str(payload).unwrap_or_else(|_| T::default())
}

pub enum Cache {
    HIT,
    MISS,
    SET,
    FAILED,
    SNK,
}
impl Display for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SET => write!(f, "cache set successfully"),
            Self::HIT => write!(f, "cache hit"),
            Self::MISS => write!(f, "cache miss"),
            Self::FAILED => write!(f, "cache update failed"),
            Self::SNK => write!(f, "cache set with non-existent key"),
        }
    }
}

impl Cache {
    pub fn log(&self, key: &str) {
        match self {
            Self::FAILED => log::error!("{}: {}", key, self),
            _ => log::info!("{}: {}", key, self),
        }
    }
}

pub fn srilankan_time() -> DateTime<FixedOffset> {
    let sri_lanka_offset = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
    Utc::now().with_timezone(&sri_lanka_offset)
}

pub fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

pub async fn cloudflare_verify(token: &str, ip: &str) -> bool {
    let client = Client::new();
    let key = Ulid::new().to_string();

    let response = match client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&[
            ("secret", &*ENV.turnstile_site_secret),
            ("response", token),
            ("remoteip", ip),
            ("idempotency_key", &key),
        ])
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => {
            log::error!("cloudflare verification failed [ip: {}] : {:?}", ip, err);
            return false;
        }
    };

    let json: serde_json::Value = response.json().await.unwrap_or_default();
    let success = json
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !success {
        log::error!("cloudflare verification failed [ip: {}] : {:?}", ip, json);
        return false;
    }

    log::info!("cloudflare verification success [ip: {}]", ip);
    true
}

pub fn governer_conf() -> Arc<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>> {
    Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(100)
            .key_extractor(SmartIpKeyExtractor)
            .error_handler(|err: GovernorError| match err {
                GovernorError::TooManyRequests { wait_time, headers } => {
                    if headers.is_none() {
                        return (
                            StatusCode::TOO_MANY_REQUESTS,
                            format!("Rate limit exceeded, try again in {} seconds", wait_time),
                        )
                            .into_response();
                    }
                    let headers = headers.unwrap();

                    let wants_json = headers
                        .get(header::ACCEPT)
                        .and_then(|value| value.to_str().ok())
                        .map(|accept| accept.contains("application/json"))
                        .unwrap_or(false);

                    if wants_json {
                        return (
                            StatusCode::TOO_MANY_REQUESTS,
                            [(header::CONTENT_TYPE, "application/json")],
                            Json(json!({
                                "error": "Rate limit exceeded",
                                "wait_time": wait_time,
                            })),
                        )
                            .into_response();
                    }

                    Html(ratelimit::Tmpl::new(wait_time).render().unwrap()).into_response()
                }
                _ => Html(servererror::Tmpl::default().render().unwrap()).into_response(),
            })
            .finish()
            .unwrap(),
    )
}

pub fn get_domain() -> String {
    let domain = &*ENV.domain;
    match domain.ends_with("/") {
        true => domain.trim_end_matches('/').to_string(),
        false => domain.to_string(),
    }
}

pub fn headers(map: HashMap<HeaderName, &'static str>) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in map {
        headers.insert(key, value.parse().unwrap());
    }
    headers
}
