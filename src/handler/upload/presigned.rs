use crate::{
    config::{ENV, state::AppState},
    error::{AppError, JsonError},
    model::r2::{self},
    util::cloudflare_verify,
};
use aws_sdk_s3::presigning::PresigningConfig;
use axum::{
    Json,
    extract::{ConnectInfo, State},
    response::IntoResponse,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde_json::json;
use std::{net::SocketAddr, time::Duration};
use validator::Validate;

pub async fn run(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    Json(payload): Json<r2::Payload>,
) -> Result<impl IntoResponse, JsonError> {
    let ip = addr.ip().to_string();

    let mut is_authorized = false;
    if let Some(TypedHeader(auth)) = authorization
        && auth.token() == &*ENV.turnstile_site_secret
    {
        is_authorized = true;
    }

    if !is_authorized && !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return Err(AppError::captcha("captcha failed").into());
    }

    payload.validate()?;
    if payload.password != (*ENV.admin_password) {
        return Err(AppError::unauthorized("password is incorrect").into());
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
