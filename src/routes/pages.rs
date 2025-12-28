use axum::Router;
use axum::routing::get;

use crate::config::state::AppState;
use crate::pages::{about, blog, index, key, post, resume, short, upload};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index::render))
        .route("/about", get(about::render))
        .route("/blog/{page}", get(blog::paginated))
        .route("/{key}", get(key::render))
        .route("/get-resume", get(resume::render))
        .nest(
            "/posts",
            Router::new()
                .route("/{slug}", get(post::view::render))
                .route("/{slug}/edit", get(post::edit::render))
                .route("/add", get(post::add::render))
                .route("/del", get(post::delete::render)),
        )
        .nest(
            "/upload",
            Router::new()
                .route("/", get(upload::index::render))
                .route("/storage", get(upload::storage::render))
                .route("/cdn", get(upload::cdn::render))
                .route("/list", get(upload::list::render))
                .route("/delete", get(upload::delete::render)),
        )
        .nest(
            "/short",
            Router::new()
                .route("/", get(short::index::render))
                .route("/add", get(short::add::render))
                .route("/del", get(short::del::render))
                .route("/list", get(short::list::render)),
        )
}
