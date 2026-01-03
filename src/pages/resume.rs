use crate::{
    config::state::AppState,
    database,
    error::{AppError, HtmlError},
};
use axum::{
    extract::State,
    http::{HeaderMap, header},
    response::IntoResponse,
};

pub async fn render(State(state): State<AppState>) -> Result<impl IntoResponse, HtmlError> {
    let custom_filename = "Vinuka_Kodituwakku_Resume.pdf";

    let short = database::short::get(&state.db, "resume")
        .await
        .map_err(AppError::from_database_error)?;

    let mut url = short.long_url;
    // https://docs.google.com/document/d/1uOOD2jIkuyOUloVVQ6V5rGUtiImiurZ1kHKShmLxTjc/edit?usp=sharing
    if url.is_empty() || !url.contains("docs.google.com/document/d/") {
        return Err(AppError::not_found("page not found").into());
    }
    let id = url
        .strip_prefix("https://docs.google.com/document/d/")
        .or_else(|| url.split("/d/").nth(1))
        .and_then(|s| s.split('/').next())
        .ok_or_else(|| AppError::not_found("page not found"))?;
    url = format!(
        "https://docs.google.com/document/d/{}/export?format=pdf",
        id
    );

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::not_found("page not found").into());
    }

    let bytes = resp.bytes().await?;
    let mut headers = HeaderMap::new();

    headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", custom_filename)
            .parse()
            .unwrap(),
    );

    Ok((headers, bytes))
}
