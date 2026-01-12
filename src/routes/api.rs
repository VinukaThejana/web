use crate::config::state::AppState;
use crate::handler::{health, metadata, short, upload};
use axum::Router;
use axum::routing::{get, post};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .nest(
            "/upload",
            Router::new()
                .route("/storage", post(upload::presigned::run))
                .route("/cdn", post(upload::cdn::run)),
        )
        .nest(
            "/short",
            Router::new().route("/add", post(short::add_api::run)),
        )
        .nest(
            "/metadata",
            Router::new().route("/social", post(metadata::social::run)),
        )
}
