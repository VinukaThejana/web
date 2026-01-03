use super::{CaptchaFailed, Failed, Invalid, Okay};
use crate::{
    config::{ENV, state::AppState},
    database,
    error::{AppError, HtmlError},
    model::short::AddShort,
    util::{cloudflare_verify, html},
};
use axum::{
    Form,
    extract::{ConnectInfo, State},
    response::IntoResponse,
};
use std::net::SocketAddr;
use validator::Validate;

const FORM_ID: &str = "shorten-link-form";

pub async fn run(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<AddShort>,
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

    if let Err(e) = database::short::add(
        &state.db,
        &payload.long_url,
        &payload.key,
        &payload.description,
    )
    .await
    .map_err(AppError::from_database_error)
    {
        log::error!("failed to add key to the database: {}", e);
        match e {
            AppError::UniqueViolation { .. } => {
                return html::render(Invalid::new(FORM_ID, "key is already in use"));
            }
            _ => return html::render(Failed::default()),
        }
    }

    html::render(Okay::new(
        "Shortned",
        "Your link has been successfully shortened. You can now use it to access your content easily.",
    ))
}
