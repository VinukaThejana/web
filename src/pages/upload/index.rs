use axum::response::{IntoResponse, Redirect};

pub async fn render() -> impl IntoResponse {
    Redirect::permanent("/upload/list")
}
