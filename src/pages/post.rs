use crate::{
    cache::post::{gck_for_slug, gct_for_slug},
    config::{ENV, state::AppState},
    database,
    error::AppError,
    util::{Cache, NON_EXISTENT_KEY, parser::cmark},
};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use chrono::{DateTime, Utc};
use redis::RedisResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Template, Default)]
#[template(path = "post.html")]
pub struct Tmpl<'a> {
    pub title: &'a str,
    pub image_url: &'a str,
    pub summary: &'a str,
    pub content: &'a str,
    pub url: &'a str,
    pub keywords: &'a str,
    pub date: &'a str,
    pub word_count: usize,
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
    fn from_cache(payload: &str) -> Self {
        serde_json::from_str(payload).unwrap_or(Self::default())
    }

    fn to_cache(&self) -> String {
        if self.title.is_empty() && self.content.is_empty() {
            return NON_EXISTENT_KEY.to_string();
        }
        serde_json::to_string(self).unwrap_or(NON_EXISTENT_KEY.to_string())
    }
}
impl From<entity::post::Model> for PostCache {
    fn from(value: entity::post::Model) -> Self {
        let datetime = DateTime::<Utc>::from_timestamp(value.date.into(), 0).unwrap();
        let date = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

        Self {
            title: value.title,
            image_url: value.photo_url,
            summary: value.summary,
            content: value.content,
            url: value.slug,
            keywords: value.tags,
            date,
        }
    }
}

pub async fn render(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let ck = gck_for_slug(&slug);
    let ct = gct_for_slug();

    let mut conn = state.get_redis_conn().await.map_err(AppError::Other)?;
    let content: Option<String> = redis::cmd("GET")
        .arg(&ck)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    if let Some(content) = content {
        Cache::HIT.log(&ck);

        if content == NON_EXISTENT_KEY {
            return Err(AppError::NotFound(anyhow::anyhow!(
                "post with slug {} not found",
                slug
            )));
        }

        let post_cache = PostCache::from_cache(&content);
        let post = Tmpl {
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

    Cache::MISS.log(&ck);

    let post = database::post::get_by_slug(&state.db, &slug).await;
    if let Err(e) = post {
        tokio::spawn(async move {
            let result: RedisResult<()> = redis::cmd("SET")
                .arg(&ck)
                .arg(NON_EXISTENT_KEY)
                .arg("EX")
                .arg(ct)
                .query_async(&mut conn)
                .await;
            if let Err(e) = result {
                log::error!("{:?}", e);
                Cache::FAILED.log(&ck);
            }
            Cache::SET.log(&ck);
        });

        log::error!("post not found: {:?}", e);
        return Err(AppError::NotFound(e.into()));
    }

    let post = post.unwrap();
    let post: PostCache = post.into();
    let domain = if (*ENV.domain).ends_with("/") {
        format!("{}{}", &*ENV.domain, &slug)
    } else {
        format!("{}/{}", &*ENV.domain, &slug)
    };

    let cache = post.to_cache();
    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(&ck)
            .arg(&cache)
            .arg("EX")
            .arg(ct)
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&ck);
        }
        Cache::SET.log(&ck);
    });

    let post = Tmpl {
        title: &post.title,
        image_url: &post.image_url,
        summary: &post.summary,
        content: &cmark(&post.content),
        url: &domain,
        keywords: &post.keywords,
        date: &post.date,
        word_count: words_count::count(&post.content).words,
    };
    Ok(Html(post.render().unwrap()))
}
