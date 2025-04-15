use askama::Template;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse};

#[derive(Debug, Template, Default)]
#[template(path = "429.html")]
pub struct Tmpl {
    wait_time: u64,
}
impl Tmpl {
    pub fn new(wait_time: u64) -> Self {
        Self { wait_time }
    }
}

pub async fn render(wait_time: u64) -> impl IntoResponse {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [(header::CONTENT_TYPE, "text/html")],
        Html(Tmpl::new(wait_time).render().unwrap()),
    )
}
