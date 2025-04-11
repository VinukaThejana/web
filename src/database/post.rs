use std::time::Duration;

use redis::RedisResult;
use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    entity::*,
};

use crate::{
    config::{ENV, state::AppState},
    error::AppError,
    util,
};

pub async fn get(db: &DatabaseConnection) -> Result<Vec<entity::post::Model>, DbErr> {
    let posts = entity::post::Entity::find()
        .order_by_desc(entity::post::Column::Id)
        .limit(util::POST_LIMIT as u64)
        .all(db)
        .await?;

    Ok(posts)
}

#[derive(PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

pub async fn get_by_page(
    db: &DatabaseConnection,
    page: u64,
    order: Order,
    has_initial_data: bool,
) -> Result<Vec<entity::post::Model>, DbErr> {
    let limit: u64 = util::POST_LIMIT.try_into().unwrap();
    let offset: u64 = (if has_initial_data { limit } else { 0 } + ((page - 1) * limit));

    let posts = if order == Order::Desc {
        entity::post::Entity::find()
            .order_by_desc(entity::post::Column::Id)
            .offset(offset)
            .limit((util::POST_LIMIT + 1) as u64)
            .all(db)
            .await?
    } else {
        entity::post::Entity::find()
            .order_by_asc(entity::post::Column::Id)
            .offset(offset)
            .limit((util::POST_LIMIT + 1) as u64)
            .all(db)
            .await?
    };

    Ok(posts)
}

pub async fn get_by_slug(
    db: &DatabaseConnection,
    slug: &str,
) -> Result<entity::post::Model, DbErr> {
    let post = entity::post::Entity::find()
        .filter(entity::post::Column::Slug.eq(slug.to_lowercase()))
        .one(db)
        .await?;
    let post = post.ok_or(DbErr::RecordNotFound(String::from("post not found")))?;

    Ok(post)
}

pub async fn get_total_posts(state: AppState) -> Result<u64, AppError> {
    let ck = format!("{}:blog:totalpages", &ENV.redis_schema);

    let mut conn = state.get_redis_conn().await.map_err(AppError::Other)?;
    let result: Option<u64> = redis::cmd("GET")
        .arg(&ck)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    if result.is_some() {
        log::info!("cache hit");
        return Ok(result.unwrap());
    }

    log::info!("cache miss");

    let total_posts = entity::post::Entity::find()
        .count(&state.db)
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
            .arg(&ck)
            .arg(total_posts)
            .arg("EX")
            .arg(Duration::from_secs(30 * 24 * 60 * 60).as_secs())
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("cache set failed : {:?}", e);
            return;
        }

        log::info!("cache set successfully");
    });

    Ok(total_posts)
}
