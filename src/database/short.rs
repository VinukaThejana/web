use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect, entity::*};

pub async fn is_key_valid(db: &DatabaseConnection, key: &str) -> Result<bool, DbErr> {
    let key = entity::short::Entity::find()
        .filter(entity::short::Column::Key.eq(key))
        .limit(1)
        .one(db)
        .await?;

    if key.is_some() { Ok(false) } else { Ok(true) }
}
