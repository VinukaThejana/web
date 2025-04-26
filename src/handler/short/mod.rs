use crate::{
    config::{ENV, state::AppState},
    database,
    error::{AppError, HtmlError},
    model::short::{AddShort, ShortKey},
    util::cloudflare_verify,
};
use askama::Template;
use axum::{
    Form,
    extract::{ConnectInfo, State},
    response::{Html, IntoResponse},
};
use std::net::SocketAddr;
use validator::Validate;

#[derive(Debug, Default, Template)]
#[template(path = "components/short/key.html")]
pub struct ValidateKey<'a> {
    pub value: &'a str,
    pub message: &'a str,
    pub message_type: &'a str,
}
impl<'a> ValidateKey<'a> {
    pub fn okay(key: &'a str) -> Self {
        Self {
            value: key,
            message: "Key used is valid",
            message_type: "info",
        }
    }

    pub fn invalid(key: &'a str, message: &'a str) -> Self {
        Self {
            value: key,
            message,
            message_type: "error",
        }
    }
}

pub async fn verify(
    State(state): State<AppState>,
    Form(payload): Form<ShortKey>,
) -> Result<impl IntoResponse, HtmlError> {
    if let Err(e) = payload.validate() {
        return Ok(Html(
            ValidateKey::invalid(&payload.key, &AppError::Validation(e).to_string())
                .render()
                .unwrap(),
        ));
    }

    let Ok(is_valid) = database::short::is_key_valid(&state.db, &payload.key).await else {
        return Ok(Html(
            ValidateKey::invalid(&payload.key, "Failed to check key validity")
                .render()
                .unwrap(),
        ));
    };

    if !is_valid {
        return Ok(Html(
            ValidateKey::invalid(&payload.key, "Key is already in use")
                .render()
                .unwrap(),
        ));
    }

    Ok(Html(ValidateKey::okay(&payload.key).render().unwrap()))
}

#[derive(Debug, Default, Template)]
#[template(path = "components/short/success.html")]
pub struct AddOkay {}

#[derive(Debug, Default, Template)]
#[template(path = "components/short/invalid.html")]
pub struct AddInvalid<'a> {
    pub message: &'a str,
}
impl<'a> AddInvalid<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }
}

#[derive(Debug, Default, Template)]
#[template(path = "components/short/failed.html")]
pub struct AddFailed {}

#[derive(Debug, Default, Template)]
#[template(path = "components/short/captcha.html")]
pub struct AddCaptcha {}

pub async fn add(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<AddShort>,
) -> Result<impl IntoResponse, HtmlError> {
    let ip = addr.ip().to_string();

    if !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return Ok(Html(AddCaptcha::default().render().unwrap()));
    }

    if let Err(e) = payload.validate() {
        return Ok(Html(
            AddInvalid::new(&AppError::Validation(e).to_string())
                .render()
                .unwrap(),
        ));
    }

    if payload.password != (*ENV.admin_password) {
        return Ok(Html(
            AddInvalid::new("password is incorrect").render().unwrap(),
        ));
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
            AppError::UniqueViolation(_) => {
                return Ok(Html(
                    AddInvalid::new("key is already in use").render().unwrap(),
                ));
            }
            _ => return Ok(Html(AddFailed::default().render().unwrap())),
        }
    }

    Ok(Html(AddOkay::default().render().unwrap()))
}
