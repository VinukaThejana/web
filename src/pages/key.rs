use crate::{
    config::state::AppState,
    database,
    error::{AppError, HtmlError},
    util::SOCIALS,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

pub async fn render(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, HtmlError> {
    if key.is_empty() {
        return Err(AppError::not_found(format!("{} : page not found", key)).into());
    }

    // INFO: check weather social redirects are available with the given key
    // they take a high priority over the rest of the keys
    let social = key.to_lowercase().trim().to_owned();
    let url = SOCIALS.get(&social);
    if let Some(url) = url {
        return Ok(Redirect::permanent(url));
    }

    // INFO: then check the database for the key
    let short = database::short::get(state.db().await, &key)
        .await
        .map_err(AppError::from_database_error)?;

    tokio::spawn(async move {
        database::short::increase_views(state.db().await, &key)
            .await
            .map_err(AppError::from_database_error)
            .unwrap_or_else(|e| {
                log::error!("failed to increase views for key({key}) : {}", e);
            });
    });

    Ok(Redirect::permanent(&short.long_url))
}
