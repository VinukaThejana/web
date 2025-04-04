use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Debug, Template, Default)]
#[template(path = "404.html")]
pub struct NotFound {}

pub async fn render() -> impl IntoResponse {
    Html(NotFound::default().render().unwrap())
}
