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
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
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

    let governer_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(20)
            .finish()
            .unwrap(),
    );
    let governer_limiter = governer_conf.limiter().clone();
    let interval = Duration::from_secs(60);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            info!("rate limiting storage size: {}", governer_limiter.len());
            governer_limiter.retain_recent();
        }
    });

    let app = Router::new()
        .route("/", get(pages::index::render))
        .route("/about", get(pages::about::render))
        .route("/blog/{page}", get(pages::blog::paginated))
        .route("/{social}", get(pages::social::render))
        .route("/posts/{slug}", get(pages::post::render))
        .route("/add-post", get(pages::addpost::render))
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
                            .route("/home/load-more", post(handler::posts::home::load_more))
                            .route("/add", post(handler::posts::add)),
                    )
                    .route("/contact/send", post(handler::contact::send_msg)),
            ),
        )
        .route("/sitemap.xml", get(handler::site_xml))
        .route("/favicon.ico", get(handler::favicon))
        .route("/apple-touch-icon.png", get(handler::apple_icon))
        .route(
            "/apple-touch-icon-precomposed.png",
            get(handler::apple_icon_precompressed),
        )
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400"),
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
                )
                .layer(GovernorLayer {
                    config: governer_conf,
                }),
        )
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
