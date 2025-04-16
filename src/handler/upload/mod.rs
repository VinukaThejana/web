use crate::{
    config::state::AppState,
    error::{AppError, JsonError},
    model::r2::Payload,
    util::cloudflare_verify,
};
use axum::{
    Json,
    extract::{ConnectInfo, State},
    response::IntoResponse,
};
use std::net::SocketAddr;

pub async fn presigned(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(payload): Json<Payload>,
) -> Result<impl IntoResponse, JsonError> {
    let ip = addr.ip().to_string();

    if !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return Ok(Html(PostCaptchaFailed::default().render().unwrap()));
    }
    Ok(())
}
