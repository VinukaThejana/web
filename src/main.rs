use axum::{
    Router,
    http::{Method, StatusCode, header},
    routing::get,
};
use lambda_http::{Error, run};
use portfolio::{
    config::{log, state::AppState},
    handler,
    pages::{self},
    routes::{api, components},
};
use portfolio::{routes, util::governer_conf};
use std::{net::SocketAddr, time::Duration};
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

#[tokio::main]
async fn main() -> anyhow::Result<(), Error> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");
    log::setup();

    let state = AppState::new().await;
    let governer_conf = governer_conf();

    let mut app = Router::new()
        .merge(routes::pages::routes())
        .fallback(pages::status::notfound::render)
        .nest(
            "/api",
            Router::new()
                .merge(api::routes())
                .nest("/components", Router::new().merge(components::routes())),
        )
        .route("/sitemap.xml", get(handler::site_xml))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    Duration::from_secs(10),
                ))
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

    if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
        run(app).await
    } else {
        use axum::handler::Handler;

        let serve_dir = tower_http::services::ServeDir::new("public")
            .not_found_service(pages::status::notfound::render.with_state(state.clone()));

        app = app
            .nest_service(
                "/assets",
                tower_http::services::ServeDir::new("public/assets"),
            )
            .fallback_service(serve_dir);

        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        ::log::info!("listening on http://{}", addr);

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await?;

        Ok(())
    }
}
