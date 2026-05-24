use super::{CaptchaFailed, Failed, Invalid, Okay};
use crate::{
    config::{ENV, state::AppState},
    database,
    error::{AppError, HtmlError},
    model::post::DelPost,
    util::{
        cloudflare_verify,
        html::{self},
    },
};
use axum::{
    Form,
    extract::{ConnectInfo, State},
    response::IntoResponse,
};
use redis::RedisResult;
use std::net::SocketAddr;
use validator::Validate;

const FORM_ID: &str = "del-post-form";

pub async fn run(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<DelPost>,
) -> Result<impl IntoResponse, HtmlError> {
    let ip = addr.ip().to_string();

    if !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return html::render(CaptchaFailed::new(FORM_ID));
    }

    if let Err(e) = payload.validate() {
        return html::render(Invalid::new(FORM_ID, &AppError::Validation(e).to_string()));
    }

    if payload.password != (*ENV.admin_password) {
        return html::render(Invalid::new(FORM_ID, "password is incorrect"));
    }

    let mut conn = state.redis().await?;
    if let Err(e) = database::post::del_by_slug(state.db().await, &payload.slug)
        .await
        .map_err(AppError::from_database_error)
    {
        log::error!("failed to delete post: {}", e);
        return html::render(Failed::default());
    }

    let result: RedisResult<()> = redis::pipe()
        .flushdb()
        .ignore()
        .query_async(&mut conn)
        .await;
    if let Err(err) = result {
        log::error!("failed to flush the redis database: {}", err);
        return html::render(Failed::default());
    }

    html::render(Okay::new(
        "Deleted",
        "Your post has been successfully deleted from the database.",
    ))
}
