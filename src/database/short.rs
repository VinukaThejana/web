use prelude::Expr;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect, entity::*};

pub async fn is_key_valid(db: &DatabaseConnection, key: &str) -> Result<bool, DbErr> {
    let key = entity::short::Entity::find()
        .filter(entity::short::Column::Key.eq(key))
        .limit(1)
        .one(db)
        .await?;

    if key.is_some() { Ok(false) } else { Ok(true) }
}

pub async fn add(
    db: &DatabaseConnection,
    url: &str,
    key: &str,
    description: &str,
) -> Result<(), DbErr> {
    let short = entity::short::ActiveModel {
        long_url: Set(url.to_owned()),
        key: Set(key.to_owned()),
        description: Set(description.to_owned()),
        ..Default::default()
    };
    short.insert(db).await?;

    Ok(())
}

pub async fn get(db: &DatabaseConnection, key: &str) -> Result<entity::short::Model, DbErr> {
    let short = entity::short::Entity::find()
        .filter(entity::short::Column::Key.eq(key))
        .one(db)
        .await?;
    let short = short.ok_or(DbErr::RecordNotFound(String::from("short not found")))?;

    Ok(short)
}

pub async fn increase_views(db: &DatabaseConnection, key: &str) -> Result<(), DbErr> {
    entity::short::Entity::update_many()
        .col_expr(
            entity::short::Column::Views,
            Expr::col(entity::short::Column::Views).add(1),
        )
        .filter(entity::short::Column::Key.eq(key))
        .exec(db)
        .await?;

    Ok(())
}
