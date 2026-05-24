use crate::{
    config::state::AppState,
    database::{self, post::Order},
    error::HtmlError,
    model::post::{Post, ToPosts},
    util::{self, html},
};
use askama::Template;
use axum::{Form, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Default, Debug, Template)]
#[template(path = "components/article/load-more.html")]
pub struct LoadMore {
    pub posts: Vec<Post>,
    pub next_page: u64,
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct Payload {
    #[validate(range(min = 1))]
    pub page: u64,
}

pub async fn run(
    State(state): State<AppState>,
    Form(payload): Form<Payload>,
) -> Result<impl IntoResponse, HtmlError> {
    let page = payload.validate().map(|_| payload.page).unwrap_or(1);
    let mut load_more = LoadMore::default();

    let mut posts = database::post::get_by_page(state.db().await, page, Order::Asc, true)
        .await
        .unwrap_or(vec![])
        .to_posts();

    load_more.next_page = page + 1;
    load_more.has_more = posts.len() > util::POST_LIMIT;
    posts.pop();
    load_more.posts = posts;

    html::render(load_more)
}
