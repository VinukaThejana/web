use ::log::info;
use axum::{
    Router,
    http::{HeaderValue, Method, header},
    routing::{get, post},
};
use portfolio::util::governer_conf;
use portfolio::{
    config::{ENV, log, state::AppState},
    handler,
    pages::{self},
    util,
};
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
        .route("/", get(pages::index::render))
        .route("/about", get(pages::about::render))
        .route("/blog/{page}", get(pages::blog::paginated))
        .route("/{key}", get(pages::key::render))
        .nest(
            "/posts",
            Router::new()
                .route("/{slug}", get(pages::post::view::render))
                .route("/{slug}/edit", get(pages::post::edit::render))
                .route("/add", get(pages::post::add::render))
                .route("/del", get(pages::post::delete::render)),
        )
        .nest(
            "/upload",
            Router::new()
                .route("/", get(get(pages::upload::index::render)))
                .route("/storage", get(pages::upload::storage::render))
                .route("/cdn", get(pages::upload::cdn::render))
                .route("/list", get(pages::upload::list::render))
                .route("/delete", get(pages::upload::delete::render)),
        )
        .nest(
            "/short",
            Router::new()
                .route("/", get(pages::short::index::render))
                .route("/add", get(pages::short::add::render))
                .route("/del", get(pages::short::del::render))
                .route("/list", get(pages::short::list::render)),
        )
        .fallback(pages::status::notfound::render)
        .nest(
            "/api",
            Router::new()
                .route("/health", get(handler::health))
                .nest(
                    "/upload",
                    Router::new()
                        .route("/storage", post(handler::upload::presigned))
                        .route("/cdn", post(handler::upload::cdn)),
                )
                .nest(
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
                                .route("/add", post(handler::posts::add))
                                .route("/edit", post(handler::posts::edit))
                                .route("/del", post(handler::posts::delete)),
                        )
                        .nest(
                            "/short",
                            Router::new()
                                .route("/verify", post(handler::short::verify))
                                .route("/add", post(handler::short::add))
                                .route("/del", post(handler::short::delete)),
                        )
                        .nest(
                            "/upload",
                            Router::new().route("/del", post(handler::upload::delete)),
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
