use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Debug, Template)]
#[template(path = "upload/cdn.html")]
pub struct Tmpl<'a> {
    pub active: &'a str,
}
impl Default for Tmpl<'_> {
    fn default() -> Self {
        Self { active: "cdn" }
    }
}

pub async fn render() -> impl IntoResponse {
    Html(Tmpl::default().render().unwrap())
}
