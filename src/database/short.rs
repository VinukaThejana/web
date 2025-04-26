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
