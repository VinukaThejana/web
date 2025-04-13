use chrono::Duration;
use redis::RedisResult;

use crate::{
    config::{ENV, state::AppState},
    database,
    error::AppError,
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

pub async fn gtp(state: AppState, force: bool) -> Result<u64, AppError> {
    let cp = "total-pages";

    if !force {
        let mut conn = state.get_redis_conn().await.map_err(AppError::Other)?;
        let result: Option<u64> = redis::cmd("GET")
            .arg(gck_for_total())
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Other(e.into()))?;
        if let Some(result) = result {
            Cache::HIT.log(cp);
            return Ok(result);
        }

        Cache::MISS.log(cp);
    }

    let tp = database::post::get_total_posts(&state.db)
        .await
        .map_err(AppError::from_database_error)?;
    tokio::spawn(async move {
        let conn = state.get_redis_conn().await.map_err(AppError::Other);
        if let Err(e) = conn {
            log::error!("failed to aqquire redis connection : {:?}", e);
            return;
        }
        let mut conn = conn.unwrap();
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(gck_for_total())
            .arg(tp)
            .arg("EX")
            .arg(gct_for_total())
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(cp);
            return;
        }

        Cache::SET.log(cp);
    });

    Ok(tp)
}
