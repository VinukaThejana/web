pub mod newsletter;

use axum::{
    Json,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;

pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "status": "ok",
            "message": "service is up and running",
        })),
    )
}
