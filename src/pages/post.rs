use crate::{
    config::{ENV, state::AppState},
    database,
    error::AppError,
    util::parser::cmark,
};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use chrono::{DateTime, Utc};
use redis::RedisResult;
use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};

const NON_EXISTENT_POST: &str = "non-existent-post";
const POSTS_DIR: &str = "posts";

#[derive(Debug, Template, Default)]
#[template(path = "post.html")]
pub struct Post<'a> {
    pub title: &'a str,
    pub image_url: &'a str,
    pub summary: &'a str,
    pub content: &'a str,
    pub url: &'a str,
    pub keywords: &'a str,
    pub date: &'a str,
    pub word_count: usize,
}

pub fn get_cache_key(slug: &str) -> String {
    format!("{}:post:{}", &ENV.redis_schema, slug)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PostCache {
    title: String,
    image_url: String,
    summary: String,
    content: String,
    url: String,
    keywords: String,
    date: String,
}
impl PostCache {
    fn new(
        title: String,
        image_url: String,
        summary: String,
        content: String,
        url: String,
        keywords: String,
        date: String,
    ) -> Self {
        Self {
            title,
            image_url,
            summary,
            content,
            url,
            keywords,
            date,
        }
    }

    fn from_cache(payload: &str) -> Self {
        serde_json::from_str(payload).unwrap_or(Self::default())
    }

    fn to_cache(&self) -> String {
        if self.title.is_empty() && self.content.is_empty() {
            return NON_EXISTENT_POST.to_string();
        }

        serde_json::to_string(self).unwrap_or(NON_EXISTENT_POST.to_string())
    }
}

pub async fn render(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.get_redis_conn().await.map_err(AppError::Other)?;
    let content: Option<String> = redis::cmd("GET")
        .arg(get_cache_key(&slug))
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    if let Some(content) = content {
        if content == NON_EXISTENT_POST {
            return Err(AppError::NotFound(anyhow::anyhow!(
                "post with slug {} not found",
                slug
            )));
        }

        log::info!("cache hit for post: {}", slug);
        let post_cache = PostCache::from_cache(&content);
        let post = Post {
            title: &post_cache.title,
            image_url: &post_cache.image_url,
            summary: &post_cache.summary,
            content: &cmark(&post_cache.content),
            url: &post_cache.url,
            keywords: &post_cache.keywords,
            date: &post_cache.date,
            word_count: words_count::count(&post_cache.content).words,
        };
        return Ok(Html(post.render().unwrap()));
    }

    log::info!("cache miss for post: {}", slug);

    let markdown = fs::read_to_string(format!("{}/{}.md", POSTS_DIR, slug));
    if let Err(e) = markdown {
        tokio::spawn(async move {
            let conn = state.get_redis_conn().await.map_err(AppError::Other);
            if let Err(e) = conn {
                log::error!("failed to get redis connection: {:?}", e);
                return;
            }
            let mut conn = conn.unwrap();

            let result: RedisResult<()> = redis::cmd("SET")
                .arg(get_cache_key(&slug))
                .arg(NON_EXISTENT_POST)
                .arg("EX")
                .arg(Duration::from_secs(24 * 60 * 60).as_secs())
                .query_async(&mut conn)
                .await;
            if let Err(e) = result {
                log::error!("failed to set redis not found key: {:?}", e);
            }
            log::info!("added non-existent post to cache: {}", slug);
        });

        log::error!("post not found: {:?}", e);
        return Err(AppError::NotFound(e.into()));
    }

    let markdown = markdown.unwrap();
    let post = database::post::get_by_slug(&state.db, &slug)
        .await
        .map_err(AppError::from_database_error)?;
    let datetime = DateTime::<Utc>::from_timestamp(post.date.into(), 0).unwrap();
    let date = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    let domain = if (*ENV.domain).ends_with("/") {
        format!("{}{}", &*ENV.domain, &slug)
    } else {
        format!("{}/{}", &*ENV.domain, &slug)
    };

    let post_cache = PostCache::new(
        post.title.clone(),
        post.photo_url.clone(),
        post.summary.clone(),
        markdown.clone(),
        domain.clone(),
        post.tags.clone(),
        date.clone(),
    );

    tokio::spawn(async move {
        let conn = state.get_redis_conn().await.map_err(AppError::Other);
        if let Err(e) = conn {
            log::error!("failed to get redis connection: {:?}", e);
            return;
        }
        let mut conn = conn.unwrap();

        let result: RedisResult<()> = redis::cmd("SET")
            .arg(get_cache_key(&slug))
            .arg(post_cache.to_cache())
            .arg("EX")
            .arg(Duration::from_secs(24 * 60 * 60).as_secs())
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("failed to set redis not found key: {:?}", e);
        }
        log::info!("added post to cache: {}", slug);
    });

    let post = Post {
        title: &post.title,
        image_url: &post.photo_url,
        summary: &post.summary,
        content: &cmark(&markdown),
        url: &domain,
        keywords: &post.tags,
        date: &date,
        word_count: words_count::count(&markdown).words,
    };
    Ok(Html(post.render().unwrap()))
}
