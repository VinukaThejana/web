use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Debug, Template, Default)]
#[template(path = "404.html")]
pub struct Tmpl {}

pub async fn render() -> impl IntoResponse {
    Html(Tmpl::default().render().unwrap())
}
