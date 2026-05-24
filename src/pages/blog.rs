use crate::{
    cache::{
        self,
        post::{gck_for_page, gct_for_page},
    },
    config::state::AppState,
    database::{self, post::Order},
    error::{AppError, HtmlError},
    model::post::{Post, ToPosts},
    util::{Cache, POST_LIMIT, from_cache, html, to_cache},
};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use redis::RedisResult;

#[derive(Debug, Default, Template)]
#[template(path = "blog-static.html")]
pub struct Tmpl {
    pub page: u64,
    pub total_pages: u64,
    pub posts: Vec<Post>,
}

pub async fn paginated(
    State(state): State<AppState>,
    Path(page): Path<u64>,
) -> Result<impl IntoResponse, HtmlError> {
    let tp = cache::post::gtp(state.clone(), false).await?;
    let tp = (tp as f64 / POST_LIMIT as f64).ceil() as u64;
    let mut blog = Tmpl {
        page,
        total_pages: tp,
        ..Default::default()
    };
    let ck = gck_for_page(page);
    let ct = gct_for_page();

    let mut conn = state.redis().await?;

    let payload: Option<String> = redis::cmd("GET")
        .arg(&ck)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    if let Some(payload) = payload {
        Cache::HIT.log(&ck);
        blog.posts = from_cache(&payload);
        if blog.posts.is_empty() {
            return Err(AppError::not_found(format!("No posts found for page {}", page)).into());
        }
        return html::render(blog);
    }

    Cache::MISS.log(&ck);

    let posts = database::post::get_by_page(state.db().await, page, Order::Asc, false)
        .await
        .map_err(AppError::from_database_error)?
        .to_posts();
    let payload = to_cache(&posts);
    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(&ck)
            .arg(payload)
            .arg("EX")
            .arg(ct)
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&ck);
            return;
        }
        Cache::SET.log(&ck);
    });

    if posts.is_empty() {
        return Err(AppError::not_found(format!("No posts found for page {}", page)).into());
    }

    blog.posts = posts;
    html::render(blog)
}
