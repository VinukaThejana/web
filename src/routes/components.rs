use crate::config::state::AppState;
use crate::handler::{contact, newsletter, posts, short, upload};
use axum::Router;
use axum::routing::post;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest(
            "/newsletter",
            Router::new().route("/subscribe", post(newsletter::subscribe::run)),
        )
        .nest(
            "/posts",
            Router::new()
                .route("/home/load-more", post(posts::load_more::run))
                .route("/add", post(posts::add::run))
                .route("/edit", post(posts::edit::run))
                .route("/del", post(posts::delete::run)),
        )
        .nest(
            "/short",
            Router::new()
                .route("/verify", post(short::verify::run))
                .route("/add", post(short::add::run))
                .route("/del", post(short::delete::run)),
        )
        .nest(
            "/upload",
            Router::new().route("/del", post(upload::delete::run)),
        )
        .route("/contact/send", post(contact::send_msg::run))
}
