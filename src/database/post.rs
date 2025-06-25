use crate::{
    model::post::{AddPost, PartialPost, PartialPostWithSlug},
    util::{self},
};
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

pub async fn get_by_id(db: &DatabaseConnection, id: i32) -> Result<entity::post::Model, DbErr> {
    let post = entity::post::Entity::find_by_id(id).one(db).await?;
    let post = post.ok_or(DbErr::RecordNotFound(String::from("post not found")))?;
    Ok(post)
}

pub async fn update(db: &DatabaseConnection, post: &entity::post::Model) -> Result<(), DbErr> {
    entity::post::Entity::update_many()
        .set(entity::post::ActiveModel {
            title: Set(post.title.to_owned()),
            slug: Set(post.slug.to_owned()),
            summary: Set(post.summary.to_owned()),
            photo_url: Set(post.photo_url.to_owned()),
            tags: Set(post.tags.to_owned()),
            content: Set(post.content.to_owned()),
            ..Default::default()
        })
        .filter(entity::post::Column::Id.eq(post.id))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn get_total_posts(db: &DatabaseConnection) -> Result<u64, DbErr> {
    entity::post::Entity::find().count(db).await
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

pub async fn del_by_slug(db: &DatabaseConnection, slug: &str) -> Result<(), DbErr> {
    entity::post::Entity::delete_many()
        .filter(entity::post::Column::Slug.eq(slug.to_lowercase()))
        .exec(db)
        .await?;

    Ok(())
}
