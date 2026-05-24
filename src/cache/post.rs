use chrono::{Duration, Utc};
use redis::RedisResult;

use crate::{
    config::{ENV, state::AppState},
    database,
    error::AppError,
    model::post::PartialPostWithSlug,
    util::Cache,
};

pub fn gck_for_slug(slug: &str) -> String {
    format!("{}:post:{}", &ENV.redis_schema, slug)
}

pub fn gct_for_slug() -> i64 {
    Duration::days(30).num_seconds()
}

pub fn gck_for_home() -> String {
    format!("{}:latest_posts", &ENV.redis_schema)
}
pub fn gct_for_home() -> i64 {
    Duration::days(30).num_seconds()
}

pub fn gck_for_total() -> String {
    format!("{}:blog:sum", &ENV.redis_schema)
}
pub fn gct_for_total() -> i64 {
    Duration::days(30).num_seconds()
}

pub fn gck_for_page(page: u64) -> String {
    format!("{}:blog:{}", &ENV.redis_schema, page)
}
pub fn gct_for_page() -> i64 {
    Duration::days(30).num_seconds()
}

pub fn gck_for_slugs() -> String {
    format!("{}:blog:slugs", &ENV.redis_schema)
}
pub fn gct_for_slugs() -> i64 {
    Duration::days(30).num_seconds()
}

pub fn gck_for_last_modified() -> String {
    format!("{}:blog:last-modified", &ENV.redis_schema)
}

pub async fn gtp(state: AppState, force: bool) -> Result<u64, AppError> {
    let ck = gck_for_total();
    let mut conn = state.redis().await?;

    if !force {
        let result: Option<u64> = redis::cmd("GET")
            .arg(&ck)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Other(e.into()))?;
        if let Some(result) = result {
            Cache::HIT.log(&ck);
            return Ok(result);
        }

        Cache::MISS.log(&ck);
    }

    let tp = database::post::get_total_posts(state.db().await)
        .await
        .map_err(AppError::from_database_error)?;
    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(gck_for_total())
            .arg(tp)
            .arg("EX")
            .arg(gct_for_total())
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&ck);
            return;
        }

        Cache::SET.log(&ck);
    });

    Ok(tp)
}

pub async fn get_slugs(state: AppState, force: bool) -> Result<Vec<PartialPostWithSlug>, AppError> {
    let ck = gck_for_slugs();
    let mut conn = state.redis().await?;

    if !force {
        let result: Option<String> = redis::cmd("GET")
            .arg(&ck)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Other(e.into()))?;
        if let Some(result) = result {
            Cache::HIT.log(&ck);
            let slugs: Vec<PartialPostWithSlug> =
                serde_json::from_str(&result).map_err(|e| AppError::Other(e.into()))?;
            return Ok(slugs);
        }

        Cache::MISS.log(&ck);
    }

    let slugs = database::post::get_slugs(state.db().await)
        .await
        .map_err(AppError::from_database_error)?;
    let cache = serde_json::to_string(&slugs).unwrap_or(String::from("[]"));
    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(gck_for_slugs())
            .arg(cache)
            .arg("EX")
            .arg(gct_for_total())
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&ck);
            return;
        }

        Cache::SET.log(&ck);
    });

    Ok(slugs)
}

pub async fn update_last_modified(state: AppState, page: u64, date: &str) -> Result<(), AppError> {
    let ck = gck_for_last_modified();
    let date = date.to_owned();
    let mut conn = state.redis().await?;

    let result: Option<String> = redis::cmd("GET")
        .arg(&ck)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    let mut last_modified: Vec<String> =
        serde_json::from_str(&result.unwrap_or(String::from("[]"))).unwrap_or_default();
    if last_modified.len() < page as usize {
        last_modified.resize(page.try_into().unwrap(), date.clone());
    }
    last_modified[page as usize - 1] = date.clone();

    let cache = serde_json::to_string(&last_modified).unwrap_or(String::from("[]"));
    let _: () = redis::cmd("SET")
        .arg(&ck)
        .arg(cache)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    Ok(())
}

pub async fn get_last_modified(state: AppState, tp: u64) -> Result<Vec<String>, AppError> {
    let ck = gck_for_last_modified();
    let now = Utc::now().to_rfc3339();
    let mut conn = state.redis().await?;

    let result: Option<String> = redis::cmd("GET")
        .arg(&ck)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    let mut last_modified: Vec<String> =
        serde_json::from_str(&result.unwrap_or(String::from("[]"))).unwrap_or_default();

    if last_modified.len() < tp as usize {
        last_modified.resize(tp.try_into().unwrap(), now);
    }

    let cache = serde_json::to_string(&last_modified).unwrap_or(String::from("[]"));
    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(&ck)
            .arg(cache)
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&ck);
            return;
        }
        Cache::SET.log(&ck);
    });

    Ok(last_modified)
}
