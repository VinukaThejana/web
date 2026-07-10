use super::{CaptchaFailed, Failed, Invalid, Okay};
use crate::{
    config::{ENV, state::AppState},
    database,
    error::{AppError, HtmlError},
    model::short::DelShort,
    util::{ClientIp, cloudflare_verify, html},
};
use axum::{
    Form,
    extract::State,
    response::IntoResponse,
};
use validator::Validate;

const FORM_ID: &str = "del-link-form";

pub async fn run(
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Form(payload): Form<DelShort>,
) -> Result<impl IntoResponse, HtmlError> {
    if !cloudflare_verify(state.http(), &payload.cf_turnstile_response, &ip).await {
        return html::render(CaptchaFailed::new(FORM_ID));
    }

    if let Err(e) = payload.validate() {
        return html::render(Invalid::new(FORM_ID, &AppError::Validation(e).to_string()));
    }

    if payload.password != (*ENV.admin_password) {
        return html::render(Invalid::new(FORM_ID, "password is incorrect"));
    }

    if let Err(e) = database::short::delete(state.db().await, &payload.key)
        .await
        .map_err(AppError::from_database_error)
    {
        log::error!("failed to delete key from the database: {}", e);
        match e {
            AppError::NotFound { .. } => {
                return html::render(Invalid::new(FORM_ID, "key is not found"));
            }
            _ => {
                return html::render(Failed::default());
            }
        }
    }

    html::render(Okay::new(
        "Deleted",
        "Your link has been successfully deleted from the database.",
    ))
}
