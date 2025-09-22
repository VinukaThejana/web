use crate::{
    config::{ENV, state::AppState},
    database,
    error::{AppError, JsonError},
    model::short::AddShortAPI,
};
use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde_json::json;
use validator::Validate;

pub async fn run(
    State(state): State<AppState>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    Json(payload): Json<AddShortAPI>,
) -> Result<impl IntoResponse, JsonError> {
    if !matches!(
        authorization,
        Some(TypedHeader(auth)) if auth.token() == &*ENV.turnstile_site_secret
    ) {
        return Err(AppError::Unauthorized(anyhow::anyhow!("password is incorrect")).into());
    }
    payload.validate()?;

    database::short::add(
        &state.db,
        &payload.long_url,
        &payload.key,
        &payload.description,
    )
    .await
    .map_err(AppError::from_database_error)?;

    Ok(Json(json!({
        "status": "success",
        "message": "Short URL added successfully",
        "short_url": format!("{}/{}", &*ENV.domain, &payload.key),
    })))
}
