use crate::{
    config::state::AppState,
    database,
    error::{AppError, HtmlError},
    model::short::ShortKey,
    util::html,
};
use askama::Template;
use axum::{Form, extract::State, response::IntoResponse};
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

pub async fn run(
    State(state): State<AppState>,
    Form(payload): Form<ShortKey>,
) -> Result<impl IntoResponse, HtmlError> {
    if let Err(e) = payload.validate() {
        return html::render(ValidateKey::invalid(
            &payload.key,
            &AppError::Validation(e).to_string(),
        ));
    }

    let Ok(is_valid) = database::short::is_key_valid(state.db().await, &payload.key).await else {
        return html::render(ValidateKey::invalid(
            &payload.key,
            "Failed to check key validity",
        ));
    };

    if !is_valid {
        return html::render(ValidateKey::invalid(&payload.key, "key is already in use"));
    }

    html::render(ValidateKey::okay(&payload.key))
}
