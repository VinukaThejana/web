use std::sync::Arc;

use crate::error::AppError;
use crate::util;
use dotenvy::dotenv;
use serde::Deserialize;
use std::sync::LazyLock;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct Env {
    #[validate(custom(function = "envmode::verify"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub environment: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub db_url: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub db_schema: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub redis_url: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub redis_schema: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub resend_api_key: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub domain: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub resend_audience_id: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub admin_password: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub turnstile_site_key: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub turnstile_site_secret: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub cloudflare_token_value: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub cloudflare_access_key_id: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub cloudflare_access_key_secret: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub cloudflare_endpoint: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub cloudflare_bucket_name: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub cloudinary_cloud_name: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub cloudinary_api_key: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub cloudinary_api_secret: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub gemini_api_key: Arc<str>,

    #[validate(length(min = 1, message = "must not be empty"))]
    #[serde(deserialize_with = "util::deserialize_arc_str")]
    pub gcloud_geocoding_api_key: Arc<str>,

    #[validate(range(min = 8080, max = 8090, message = "must be between 8080 and 8090"))]
    pub port: u16,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        let _ = dotenv();

        let env: Self = envy::from_env().unwrap_or_else(|e| {
            log::error!("{}, exiting ... ", e);
            std::process::exit(1);
        });

        env.validate().unwrap_or_else(|e| {
            log::error!("validation errors -> {}", AppError::Validation(e));
            log::warn!(
                "update the environment to resolve the above errors and try again, exiting ... "
            );
            std::process::exit(1);
        });

        env
    }
}

pub static ENV: LazyLock<Env> = LazyLock::new(Env::new);
