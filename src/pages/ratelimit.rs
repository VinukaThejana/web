use askama::Template;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse};

#[derive(Debug, Template, Default)]
#[template(path = "429.html")]
pub struct TooManyRequests {
    wait_time: u64,
}

pub fn render(wait_time: u64) -> impl IntoResponse {
    let template = TooManyRequests { wait_time };

    (
        StatusCode::TOO_MANY_REQUESTS,
        [(header::CONTENT_TYPE, "text/html")],
        Html(template.render().unwrap()),
    )
}
