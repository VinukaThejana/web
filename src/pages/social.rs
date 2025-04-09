use crate::{error::AppError, util::SOCIALS};
use axum::{
    extract::Path,
    response::{IntoResponse, Redirect},
};

pub async fn render(Path(social): Path<String>) -> Result<impl IntoResponse, AppError> {
    if social.is_empty() {
        return Err(AppError::NotFound(anyhow::anyhow!("page not found")));
    }
    let social = social.to_lowercase().trim().to_owned();
    let url = SOCIALS
        .get(&social)
        .ok_or_else(|| AppError::NotFound(anyhow::anyhow!("page not found")))?;

    Ok(Redirect::permanent(url))
}
