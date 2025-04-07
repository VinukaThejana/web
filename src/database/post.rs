use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryFilter, entity::*};

pub async fn get(db: &DatabaseConnection, slug: &str) -> Result<entity::post::Model, DbErr> {
    let post = entity::post::Entity::find()
        .filter(entity::post::Column::Slug.eq(slug.to_lowercase()))
        .one(db)
        .await?;
    let post = post.ok_or(DbErr::RecordNotFound(String::from("post not found")))?;

    Ok(post)
}
