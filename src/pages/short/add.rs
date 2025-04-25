use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Debug, Default, Template)]
#[template(path = "short/add.html")]
pub struct Tmpl {}

pub async fn render() -> impl IntoResponse {
    Html(Tmpl::default().render().unwrap())
}
