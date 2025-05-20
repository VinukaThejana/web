use crate::{
    config::{ENV, state::AppState},
    error::{AppError, HtmlError, JsonError},
    model::{
        cdn,
        r2::{self, DelResource},
    },
    util::{IMG_EXTENSIONS, cloudflare_verify, html},
};
use askama::Template;
use aws_sdk_s3::presigning::PresigningConfig;
use axum::{
    Form, Json,
    extract::{ConnectInfo, State},
    response::IntoResponse,
};
use serde_json::json;
use sha1::{Digest, Sha1};
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use validator::Validate;

#[derive(Debug, Default, Template)]
#[template(path = "components/upload/captcha.html")]
pub struct Captcha {}

#[derive(Debug, Default, Template)]
#[template(path = "components/upload/failed.html")]
pub struct Failed {}

#[derive(Debug, Default, Template)]
#[template(path = "components/upload/invalid.html")]
pub struct Invalid<'a> {
    pub form_id: &'a str,
    pub message: &'a str,
}
impl<'a> Invalid<'a> {
    pub fn new(form_id: &'a str, message: &'a str) -> Self {
        Self { form_id, message }
    }
}

pub async fn presigned(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(payload): Json<r2::Payload>,
) -> Result<impl IntoResponse, JsonError> {
    let ip = addr.ip().to_string();

    if !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return Err(AppError::CaptchaFailed(anyhow::anyhow!("captcha failed")).into());
    }
    payload.validate()?;
    if payload.password != (*ENV.admin_password) {
        return Err(AppError::Unauthorized(anyhow::anyhow!("password is incorrect")).into());
    }

    let presigned_url = state
        .s3
        .put_object()
        .bucket(&*ENV.cloudflare_bucket_name)
        .key(&payload.path)
        .presigned(
            PresigningConfig::expires_in(Duration::from_secs(60 * 60))
                .map_err(AppError::from_generic_error)?,
        )
        .await
        .map_err(AppError::from_generic_error)?;

    Ok(Json(json!({
        "url": presigned_url.uri().to_string(),
    })))
}

pub async fn cdn(Json(payload): Json<cdn::Payload>) -> Result<impl IntoResponse, JsonError> {
    payload.validate()?;
    if payload.password != (*ENV.admin_password) {
        return Err(AppError::Unauthorized(anyhow::anyhow!("password is incorrect")).into());
    }

    if !IMG_EXTENSIONS
        .iter()
        .any(|ext| payload.path.to_lowercase().ends_with(ext))
    {
        return Err(AppError::BadRequest(anyhow::anyhow!("only image files are allowed")).into());
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

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "https://api.cloudinary.com/v1_1/{}/image/upload",
            cloud_name
        ))
        .form(&form)
        .send()
        .await
        .map_err(AppError::from_generic_error)?;

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

#[derive(Debug, Default, Template)]
#[template(path = "components/upload/del-success.html")]
pub struct DelOkay {}

pub async fn delete(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<DelResource>,
) -> Result<impl IntoResponse, HtmlError> {
    let form_id = "delete-form";
    let ip = addr.ip().to_string();

    if !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return html::render(Captcha::default());
    }

    if let Err(e) = payload.validate() {
        return html::render(Invalid::new(form_id, &AppError::Validation(e).to_string()));
    }

    if payload.password != (*ENV.admin_password) {
        return html::render(Invalid::new(form_id, "password is incorrect"));
    }

    state
        .s3
        .delete_object()
        .bucket(&*ENV.cloudflare_bucket_name)
        .key(&payload.key)
        .send()
        .await
        .map_err(AppError::from_generic_error)?;

    html::render(DelOkay::default())
}
