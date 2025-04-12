use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Debug, Default, Template)]
#[template(path = "add-post.html")]
pub struct AddPost {}

pub async fn render() -> impl IntoResponse {
    Html(AddPost::default().render().unwrap())
}
