use ::log::info;
use axum::{
    Router,
    http::{Method, header},
    routing::get,
};
use portfolio::{
    config::{ENV, log, state::AppState},
    handler, util,
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
        .nest("/api", Router::new().route("/health", get(handler::health)))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(10)))
                .layer(
                    CorsLayer::new()
                        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                        .allow_methods([
                            Method::GET,
                            Method::POST,
                            Method::PUT,
                            Method::DELETE,
                            Method::OPTIONS,
                        ])
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
