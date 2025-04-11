use std::time::Duration;

use crate::{
    config::{ENV, state::AppState},
    database::{self, post::Order},
    error::AppError,
    model::post::{Post, ToPosts},
    util::{Cache, POST_LIMIT, from_cache, to_cache},
};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use redis::RedisResult;

#[derive(Debug, Default, Template)]
#[template(path = "blog-static.html")]
pub struct BlogPage {
    pub page: u64,
    pub total_pages: u64,
    pub posts: Vec<Post>,
}

fn gcp(page: u64) -> String {
    format!("blog-{}", page)
}
fn gck(page: u64) -> String {
    format!("{}:blog:{}", &ENV.redis_schema, page)
}

pub async fn paginated(
    State(state): State<AppState>,
    Path(page): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let tp = database::post::get_total_posts(state.clone()).await?;
    let tp = (tp as f64 / POST_LIMIT as f64).ceil() as u64;
    let mut blog = BlogPage {
        page,
        total_pages: tp,
        ..Default::default()
    };

    let mut conn = state.get_redis_conn().await.map_err(AppError::Other)?;
    let payload: Option<String> = redis::cmd("GET")
        .arg(gck(page))
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    if let Some(payload) = payload {
        Cache::HIT.log(&gcp(page));
        blog.posts = from_cache(&payload);
        if blog.posts.is_empty() {
            return Err(AppError::NotFound(anyhow::anyhow!(
                "No posts found for page {}",
                page
            )));
        }
        return Ok(Html(blog.render().unwrap()));
    }

    Cache::MISS.log(&gcp(page));

    let posts = database::post::get_by_page(&state.db, page, Order::Asc, false)
        .await
        .map_err(AppError::from_database_error)?
        .to_posts();
    let payload = to_cache(&posts);
    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(gck(page))
            .arg(payload)
            .arg("EX")
            .arg(Duration::from_secs(30 * 24 * 60 * 60).as_secs())
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&gcp(page));
            return;
        }
        Cache::SET.log(&gcp(page));
    });

    if posts.is_empty() {
        return Err(AppError::NotFound(anyhow::anyhow!(
            "No posts found for page {}",
            page
        )));
    }

    blog.posts = posts;
    Ok(Html(blog.render().unwrap()))
}
