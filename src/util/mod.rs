pub mod parser;
pub mod verify;

use crate::config::state::AppState;
use base64::prelude::*;
use serde::{Deserialize, Deserializer};
use std::sync::Arc;
use tokio::signal;

pub const AUTHOR: &str = "Vinuka Kodituwakku";
pub const AUTHOR_EMAIL: &str = "vinuka.t@icloud.com";
pub const AUTHOR_GITHUB: &str = "https://github.com/VinukaThejana";
pub const AUTHOR_TWITTER: &str = "@VinukaThejana";

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
