use ::log::info;
use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use portfolio::{
    config::{ENV, log, state::AppState},
    handler, pages, util,
};
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::setup();
    let state = AppState::new().await;

    let app = Router::new()
        .route("/", get(pages::index::render))
        .nest(
            "/api",
            Router::new().route("/health", get(handler::health)).nest(
                "/newsletter",
                Router::new().route("/subscribe", post(handler::newsletter::subscribe)),
            ),
        )
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(10)))
                .layer(
                    CorsLayer::new()
                        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                        .allow_methods([Method::GET])
                        .allow_origin(Any),
                ),
        )
        .with_state(state.clone());

    info!("up and running on : {}", &ENV.port);
    axum::serve(
        TcpListener::bind(format!("0.0.0.0:{}", &ENV.port))
            .await
            .unwrap(),
        app,
    )
    .with_graceful_shutdown(util::shutdown(state))
    .await
    .unwrap();

    anyhow::Ok(())
}
