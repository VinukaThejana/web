use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Debug, Template)]
#[template(path = "short/del.html")]
pub struct Tmpl {
    pub active: &'static str,
}
impl Default for Tmpl {
    fn default() -> Self {
        Self { active: "del" }
    }
}

pub async fn render() -> impl IntoResponse {
    Html(Tmpl::default().render().unwrap())
}
