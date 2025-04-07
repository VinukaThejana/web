use crate::error::AppError;
use axum::{
    extract::Path,
    response::{IntoResponse, Redirect},
};
use phf::phf_map;

static SOCIALS: phf::Map<&'static str, &'static str> = phf_map! {
    "github" => "https://github.com/VinukaThejana",
    "git" => "https://github.com/VinukaThejana",
    "linkedin" => "https://www.linkedin.com/in/vinukakodituwakku/",
    "in" => "https://www.linkedin.com/in/vinukakodituwakku/",
    "twitter" => "https://twitter.com/VinukaThejana",
    "x" => "https://twitter.com/VinukaThejana",
    "instagram" => "https://www.instagram.com/vinukathejana/",
    "ig" => "https://www.instagram.com/vinukathejana/",
    "facebook" => "https://www.facebook.com/vinukakodituwakku",
    "fb" => "https://www.facebook.com/vinukakodituwakku",
};

pub async fn render(Path(social): Path<String>) -> Result<impl IntoResponse, AppError> {
    if social.is_empty() {
        return Err(AppError::NotFound(anyhow::anyhow!("social not found")));
    }
    let social = social.to_lowercase().trim().to_owned();
    let url = SOCIALS
        .get(&social)
        .ok_or_else(|| AppError::NotFound(anyhow::anyhow!("social not found")))?;

    Ok(Redirect::permanent(url))
}
