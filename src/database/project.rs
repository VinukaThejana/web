use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryOrder};

pub async fn get(db: &DatabaseConnection) -> Result<Vec<entity::project::Model>, DbErr> {
    let projects = entity::project::Entity::find()
        .order_by_desc(entity::project::Column::Id)
        .all(db)
        .await?;

    Ok(projects)
}
