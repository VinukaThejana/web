use crate::{
    config::{ENV, state::AppState},
    database,
    error::AppError,
    model::post::{Post, ToPosts},
    util::{self, Cache, SOCIALS, from_cache, to_cache},
};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use redis::RedisResult;
use std::time::Duration;

pub struct Social<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub icon: &'a str,
}
impl<'a> Social<'a> {
    pub fn new(name: &'a str, url: &'a str, icon: &'a str) -> Self {
        Self { name, url, icon }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {
    pub socials: Vec<Social<'static>>,
    pub posts: Vec<Post>,
    pub has_more: bool,
}
impl Default for Index {
    fn default() -> Self {
        let socials: Vec<Social> = vec![
            Social::new("GitHub", SOCIALS.get("git").unwrap(), "fa-github"),
            Social::new("LinkedIn", SOCIALS.get("in").unwrap(), "fa-linkedin"),
            Social::new("X", SOCIALS.get("x").unwrap(), "fa-x-twitter"),
            Social::new("Facebook", SOCIALS.get("fb").unwrap(), "fa-facebook"),
            Social::new("Instagram", SOCIALS.get("ig").unwrap(), "fa-instagram"),
        ];

        Self {
            socials,
            posts: vec![],
            has_more: false,
        }
    }
}
impl Index {
    pub async fn new(posts: Vec<Post>) -> Self {
        let has_posts = posts.len() == util::POST_LIMIT;

        Self {
            posts,
            has_more: has_posts,
            ..Default::default()
        }
    }
}

const CACHE_PATH: &str = "index-posts";
fn get_cache_key() -> String {
    format!("{}:latest_posts", &ENV.redis_schema)
}

pub async fn render(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = state.get_redis_conn().await.map_err(AppError::Other);
    if let Err(e) = conn {
        log::error!("failed to get redis connection : {:?}", e);
        return Ok(Html(Index::default().render().unwrap()));
    }
    let mut conn = conn.unwrap();
    let payload: Option<String> = redis::cmd("GET")
        .arg(get_cache_key())
        .query_async(&mut conn)
        .await
        .unwrap_or(None);
    if let Some(payload) = payload {
        Cache::HIT.log(CACHE_PATH);
        let posts = from_cache(&payload);
        return Ok(Html(Index::new(posts).await.render().unwrap()));
    }

    Cache::MISS.log(CACHE_PATH);

    let posts = database::post::get(&state.db)
        .await
        .unwrap_or(vec![])
        .to_posts();

    let payload = to_cache(&posts);
    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(get_cache_key())
            .arg(payload)
            .arg("EX")
            .arg(Duration::from_secs(30 * 24 * 60 * 60).as_secs())
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(CACHE_PATH);
            return;
        }
        Cache::SET.log(CACHE_PATH);
    });

    Ok(Html(Index::new(posts).await.render().unwrap()))
}
