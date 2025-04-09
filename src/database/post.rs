use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, entity::*,
};

use crate::util;

pub async fn get(db: &DatabaseConnection) -> Result<Vec<entity::post::Model>, DbErr> {
    let posts = entity::post::Entity::find()
        .order_by_desc(entity::post::Column::Id)
        .limit(util::POST_LIMIT as u64)
        .all(db)
        .await?;

    Ok(posts)
}

pub async fn get_by_page(
    db: &DatabaseConnection,
    page: usize,
) -> Result<Vec<entity::post::Model>, DbErr> {
    let offset = (util::POST_LIMIT + ((page - 1) * util::POST_LIMIT)) as u64;

    let posts = entity::post::Entity::find()
        .order_by_desc(entity::post::Column::Id)
        .offset(offset)
        .limit((util::POST_LIMIT + 1) as u64)
        .all(db)
        .await?;

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
