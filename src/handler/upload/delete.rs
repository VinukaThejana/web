use super::{CaptchaFailed, Invalid, Okay};
use crate::{
    config::{ENV, state::AppState},
    error::{AppError, HtmlError},
    model::r2::DelResource,
    util::{cloudflare_verify, html},
};
use axum::{
    Form,
    extract::{ConnectInfo, State},
    response::IntoResponse,
};
use std::net::SocketAddr;
use validator::Validate;

const FORM_ID: &str = "delete-form";

pub async fn run(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<DelResource>,
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

    state
        .s3
        .delete_object()
        .bucket(&*ENV.cloudflare_bucket_name)
        .key(&payload.key)
        .send()
        .await
        .map_err(AppError::from_generic_error)?;

    html::render(Okay::new(
        "Deleted",
        "The resource has been successfully deleted from the storage bucket.",
    ))
}
