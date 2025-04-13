use crate::{
    cache::post::{gck_for_total, gct_for_total},
    config::state::AppState,
    error::AppError,
    model::post::{AddPost, PartialPost, PartialPostWithSlug},
    util::{self, Cache},
};
use redis::RedisResult;
use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    entity::*,
};

pub async fn get(db: &DatabaseConnection) -> Result<Vec<PartialPost>, DbErr> {
    entity::post::Entity::find()
        .select_only()
        .columns(
            entity::post::Column::iter()
                .filter(|col| !matches!(col, entity::post::Column::Content)),
        )
        .order_by_desc(entity::post::Column::Id)
        .limit(util::POST_LIMIT as u64)
        .into_model::<PartialPost>()
        .all(db)
        .await
}

pub async fn get_slugs(db: &DatabaseConnection) -> Result<Vec<PartialPostWithSlug>, DbErr> {
    entity::post::Entity::find()
        .select_only()
        .columns([
            entity::post::Column::Id,
            entity::post::Column::Slug,
            entity::post::Column::Date,
        ])
        .order_by_asc(entity::post::Column::Id)
        .into_model::<PartialPostWithSlug>()
        .all(db)
        .await
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
) -> Result<Vec<PartialPost>, DbErr> {
    let limit: u64 = util::POST_LIMIT.try_into().unwrap();
    let offset: u64 = (if has_initial_data { limit } else { 0 } + ((page - 1) * limit));

    let posts = if order == Order::Desc {
        entity::post::Entity::find()
            .select_only()
            .columns(
                entity::post::Column::iter()
                    .filter(|col| !matches!(col, entity::post::Column::Content)),
            )
            .order_by_desc(entity::post::Column::Id)
            .offset(offset)
            .limit((util::POST_LIMIT + 1) as u64)
            .into_model::<PartialPost>()
            .all(db)
            .await?
    } else {
        entity::post::Entity::find()
            .select_only()
            .columns(
                entity::post::Column::iter()
                    .filter(|col| !matches!(col, entity::post::Column::Content)),
            )
            .order_by_asc(entity::post::Column::Id)
            .offset(offset)
            .limit((util::POST_LIMIT + 1) as u64)
            .into_model::<PartialPost>()
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

pub async fn get_total_posts(state: AppState, force: bool) -> Result<u64, AppError> {
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
            .arg(gck_for_total())
            .arg(total_posts)
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

    Ok(total_posts)
}

pub async fn add(db: &DatabaseConnection, payload: &AddPost) -> Result<(), DbErr> {
    let post = entity::post::ActiveModel {
        title: Set(payload.title.to_owned()),
        slug: Set(payload.slug.to_owned()),
        summary: Set(payload.summary.to_owned()),
        photo_url: Set(payload.photo_url.to_owned()),
        tags: Set(payload.tags.to_owned()),
        content: Set(payload.content.to_owned()),
        date: Set(payload
            .date
            .try_into()
            .map_err(|_| DbErr::Custom(String::from("failed to convert date to signed")))?),
        ..Default::default()
    };
    let _: entity::post::Model = post.insert(db).await?;

    Ok(())
}
