use ::log::info;
use axum::{
    Router,
    http::{HeaderValue, Method, header},
    routing::get,
};
use portfolio::{
    config::{ENV, log, state::AppState},
    handler,
    pages::{self},
    routes::{api, components},
    util,
};
use portfolio::{routes, util::governer_conf};
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
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

    let governer_conf = governer_conf();
    let limiter = governer_conf.limiter().clone();
    let interval = Duration::from_secs(60);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            info!("rate limiting storage size: {}", limiter.len());
            limiter.retain_recent();
        }
    });

    let app = Router::new()
        .merge(routes::pages::routes())
        .fallback(pages::status::notfound::render)
        .nest(
            "/api",
            Router::new()
                .merge(api::routes())
                .nest("/components", Router::new().merge(components::routes())),
        )
        .route("/sitemap.xml", get(handler::site_xml))
        .route("/robots.txt", get(handler::robots_txt))
        .route("/favicon.ico", get(handler::favicon))
        .route("/apple-touch-icon.png", get(handler::apple_icon))
        .route(
            "/apple-touch-icon-precomposed.png",
            get(handler::apple_icon_precompressed),
        )
        .nest_service(
            "/assets",
            tower_http::services::ServeDir::new("assets")
                .precompressed_gzip()
                .precompressed_br(),
        )
        .layer(
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=86400"),
                ))
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(10)))
                .layer(
                    CorsLayer::new()
                        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                        .allow_methods([Method::GET, Method::POST])
                        .allow_origin(Any),
                ),
        )
        .layer(GovernorLayer {
            config: governer_conf,
        })
        .with_state(state.clone());

    info!("up and running on : {}", &ENV.port);
    axum::serve(
        TcpListener::bind(&format!("0.0.0.0:{}", &ENV.port))
            .await
            .unwrap(),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(util::shutdown(state))
    .await
    .unwrap();

    anyhow::Ok(())
}
