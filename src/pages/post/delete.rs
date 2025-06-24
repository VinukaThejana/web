use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Debug, Default, Template)]
#[template(path = "posts/delete.html")]
pub struct Tmpl {
    pub active: &'static str,
}

pub async fn render() -> impl IntoResponse {
    Html(Tmpl::default().render().unwrap())
}
