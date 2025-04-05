use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Debug, Template, Default)]
#[template(path = "500.html")]
pub struct InternalServerError {}

pub async fn render() -> impl IntoResponse {
    Html(InternalServerError::default().render().unwrap())
}
