use crate::{
    config::ENV,
    error::{AppError, JsonError},
    model::metadata::{GetVideoMetadata, YoutubeEmbed},
    util::metadata::is_yt_video,
};
use anyhow::Context;
use axum::{Json, response::IntoResponse};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use regex::Regex;
use serde_json::json;
use validator::Validate;

pub async fn get(
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    Json(payload): Json<GetVideoMetadata>,
) -> Result<impl IntoResponse, JsonError> {
    if !matches!(
        authorization,
        Some(TypedHeader(auth)) if auth.token() == &*ENV.turnstile_site_secret
    ) {
        return Err(AppError::unauthorized("password is incorrect").into());
    }
    payload.validate()?;

    let url = payload.url.trim();

    let mut title: Option<String> = None;
    let mut thumbnail_url: Option<String> = None;
    let mut description: Option<String> = None;

    if is_yt_video(url) {
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.youtube.com/oembed")
            .query(&[("url", url), ("format", "json")])
            .send()
            .await
            .context("failed to fetch yotube oembed data")?;
        if !response.status().is_success() {
            return Err(AppError::bad_request("video not found").into());
        }

        let metadata: YoutubeEmbed = response
            .json()
            .await
            .context("failed to parse youtube oembed data")?;

        title = Some(metadata.title);
        thumbnail_url = Some(metadata.thumbnail_url);

        let client = reqwest::Client::new();
        let html = client
            .get(url)
            .send()
            .await
            .context("failed to fetch youtube page")?
            .text()
            .await
            .context("failed to read youtube page")?;

        let re = Regex::new(r#"<meta name="description" content="([^"]+)">"#).unwrap();
        description = re
            .captures(&html)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string());
    }

    Ok(Json(json!({
        "url": payload.url,
        "title": title.unwrap_or_default(),
        "thumbnail_url": thumbnail_url.unwrap_or_default(),
        "description": description.unwrap_or_default(),
    })))
}
