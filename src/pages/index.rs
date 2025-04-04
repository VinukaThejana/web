use crate::error::AppError;
use askama::Template;
use axum::response::{Html, IntoResponse};

pub struct Social<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub icon: &'a str,
}
impl<'a> Social<'a> {
    pub fn new(name: &'a str, url: &'a str, icon: &'a str) -> Self {
        Self { name, url, icon }
    }
}

#[derive(Template, Default)]
#[template(path = "index.html")]
pub struct Index {
    pub socials: Vec<Social<'static>>,
}
impl Index {
    pub fn new() -> Self {
        let socials: Vec<Social> = vec![
            Social::new("GitHub", "https://github.com/VinukaThejana", "fa-github"),
            Social::new(
                "LinkedIn",
                "https://www.linkedin.com/in/vinukakodituwakku",
                "fa-linkedin",
            ),
            Social::new("X", "https://x.com/VinukaThejana", "fa-x-twitter"),
            Social::new(
                "Facebook",
                "https://facebook.com/vinukakodituwakku",
                "fa-facebook",
            ),
            Social::new(
                "Instagram",
                "https://www.instagram.com/vinukathejana",
                "fa-instagram",
            ),
        ];

        Self { socials }
    }
}

pub async fn render() -> Result<impl IntoResponse, AppError> {
    Ok(Html(Index::new().render().unwrap()))
}
