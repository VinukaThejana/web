use crate::{
    config::state::AppState,
    database,
    error::{AppError, HtmlError},
    model::short::ShortKey,
};
use askama::Template;
use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse},
};
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
