pub mod contact;
pub mod newsletter;
pub mod posts;

use std::fs;

use axum::{
    Json,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;

pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "status": "ok",
            "message": "service is up and running",
        })),
    )
}

pub async fn favicon() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/x-icon"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=604800"),
    );

    match fs::read("assets/icons/favicon.ico") {
        Ok(content) => (StatusCode::OK, headers, content).into_response(),
        Err(err) => {
            log::error!("error reading the favicon.ico file: {:?}", err);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

pub async fn apple_icon() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/x-icon"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=604800"),
    );

    match fs::read("assets/icons/apple-icon.png") {
        Ok(content) => (StatusCode::OK, headers, content).into_response(),
        Err(err) => {
            log::error!("error reading the apple-icon.png file: {:?}", err);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

pub async fn apple_icon_precompressed() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/x-icon"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=604800"),
    );

    match fs::read("assets/icons/apple-icon-precomposed.png") {
        Ok(content) => (StatusCode::OK, headers, content).into_response(),
        Err(err) => {
            log::error!(
                "error reading the apple-icon-precomposed.png file: {:?}",
                err
            );
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

pub async fn webmanifest() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/manifest+json"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400"),
    );

    match fs::read("assets/icons/apple-icon-precomposed.png") {
        Ok(content) => (StatusCode::OK, headers, content).into_response(),
        Err(err) => {
            log::error!(
                "error reading the apple-icon-precomposed.png file: {:?}",
                err
            );
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
