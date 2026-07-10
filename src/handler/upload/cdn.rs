use crate::{
    config::{ENV, state::AppState},
    error::{AppError, JsonError},
    model::cdn,
    util::IMG_EXTENSIONS,
};
use axum::{Json, extract::State, response::IntoResponse};
use serde_json::json;
use sha1::{Digest, Sha1};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};
use validator::Validate;

pub async fn run(
    State(state): State<AppState>,
    Json(payload): Json<cdn::Payload>,
) -> Result<impl IntoResponse, JsonError> {
    payload.validate()?;
    if payload.password != (*ENV.admin_password) {
        return Err(AppError::unauthorized("password is incorrect").into());
    }

    if !IMG_EXTENSIONS
        .iter()
        .any(|ext| payload.path.to_lowercase().ends_with(ext))
    {
        return Err(AppError::bad_request("only image files are allowed").into());
    }
    let path = match payload.path.rfind(".") {
        Some(idx) => payload.path[0..idx].to_string(),
        None => payload.path.clone(),
    };

    let api_key = &*ENV.cloudinary_api_key;
    let api_secret = &*ENV.cloudinary_api_secret;
    let cloud_name = &*ENV.cloudinary_cloud_name;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Other(e.into()))?
        .as_secs()
        .to_string();

    let mut params = BTreeMap::new();
    params.insert("public_id", path.clone());
    params.insert("timestamp", timestamp.clone());
    params.insert("use_filename", true.to_string());

    let params = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&");
    let params = format!("{}{}", params, api_secret);

    println!("params: {}", params);

    let mut hasher = Sha1::new();
    hasher.update(params.as_bytes());
    let signature = hasher.finalize();
    let signature = hex::encode(signature);

    let form = [
        ("file", payload.url),
        ("timestamp", timestamp),
        ("api_key", api_key.to_string()),
        ("public_id", path),
        ("use_filename", true.to_string()),
        ("signature", signature),
    ];

    let response = state
        .http()
        .post(format!(
            "https://api.cloudinary.com/v1_1/{}/image/upload",
            cloud_name
        ))
        .form(&form)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(
            AppError::Other(anyhow::anyhow!("failed to upload image to cloudinary")).into(),
        );
    }

    let path = if payload.path.starts_with('/') {
        payload.path[1..].to_owned()
    } else {
        payload.path
    };

    let url = format!(
        "https://res.cloudinary.com/{}/image/upload/f_auto,q_auto/v1/{}",
        cloud_name, path
    );

    Ok(Json(json!({
        "url": url,
    })))
}
