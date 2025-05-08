use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Debug, Template)]
#[template(path = "short/add.html")]
pub struct Tmpl {
    pub active: &'static str,
}
impl Default for Tmpl {
    fn default() -> Self {
        Self { active: "add" }
    }
}

pub async fn render() -> impl IntoResponse {
    Html(Tmpl::default().render().unwrap())
}
