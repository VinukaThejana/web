use crate::config::state::AppState;
use crate::handler::{health, upload};
use axum::Router;
use axum::routing::{get, post};

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health)).nest(
        "/upload",
        Router::new()
            .route("/storage", post(upload::presigned::run))
            .route("/cdn", post(upload::cdn::run)),
    )
}
