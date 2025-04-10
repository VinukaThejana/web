use ::log::info;
use axum::{
    Router,
    http::{HeaderValue, Method, header},
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
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::setup();
    let state = AppState::new().await;

    let app = Router::new()
        .route("/", get(pages::index::render))
        .route("/about", get(pages::about::render))
        .route("/{social}", get(pages::social::render))
        .route("/posts/{slug}", get(pages::post::render))
        .fallback(pages::notfound::render)
        .nest(
            "/api",
            Router::new().route("/health", get(handler::health)).nest(
                "/components",
                Router::new()
                    .nest(
                        "/newsletter",
                        Router::new().route("/subscribe", post(handler::newsletter::subscribe)),
                    )
                    .nest(
                        "/posts",
                        Router::new()
                            .route("/home/load-more", post(handler::posts::home::load_more)),
                    ),
            ),
        )
        .route("/favicon.ico", get(handler::favicon))
        .route("/apple-touch-icon.png", get(handler::apple_icon))
        .route(
            "/apple-touch-icon-precomposed.png",
            get(handler::apple_icon_precompressed),
        )
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=604800"),
        ))
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
