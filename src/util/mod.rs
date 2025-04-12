pub mod parser;
pub mod verify;

use crate::config::state::AppState;
use base64::prelude::*;
use chrono::{DateTime, FixedOffset, Utc};
use phf::phf_map;
use serde::{Deserialize, Deserializer};
use std::{fmt::Display, sync::Arc};
use tokio::signal;

pub static SOCIALS: phf::Map<&'static str, &'static str> = phf_map! {
    "github" => "https://github.com/VinukaThejana",
    "git" => "https://github.com/VinukaThejana",
    "linkedin" => "https://www.linkedin.com/in/vinukakodituwakku/",
    "in" => "https://www.linkedin.com/in/vinukakodituwakku/",
    "twitter" => "https://twitter.com/VinukaThejana",
    "x" => "https://twitter.com/VinukaThejana",
    "instagram" => "https://www.instagram.com/vinukathejana/",
    "ig" => "https://www.instagram.com/vinukathejana/",
    "facebook" => "https://www.facebook.com/vinukakodituwakku",
    "fb" => "https://www.facebook.com/vinukakodituwakku",
};

pub const AUTHOR: &str = "Vinuka Kodituwakku";
pub const AUTHOR_EMAIL: &str = "vinuka.t@icloud.com";
pub const AUTHOR_GITHUB: &str = "https://github.com/VinukaThejana";
pub const AUTHOR_TWITTER: &str = "@VinukaThejana";

pub const POST_LIMIT: usize = 10;

pub const NON_EXISTENT_KEY: &str = "non-existent-post";

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
