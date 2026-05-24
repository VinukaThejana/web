use crate::model::short::ShortModel;

pub async fn is_key_valid(db: &sqlx::PgPool, key: &str) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query("SELECT 1 FROM short WHERE key = $1")
        .bind(key)
        .fetch_optional(db)
        .await?;

    Ok(exists.is_none())
}

pub async fn add(
    db: &sqlx::PgPool,
    url: &str,
    key: &str,
    description: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO short (long_url, key, description) VALUES ($1, $2, $3)")
        .bind(url)
        .bind(key)
        .bind(description)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn get(db: &sqlx::PgPool, key: &str) -> Result<ShortModel, sqlx::Error> {
    sqlx::query_as::<_, ShortModel>(
        "SELECT id, long_url, key, description, views, created_at FROM short WHERE key = $1"
    )
    .bind(key)
    .fetch_one(db)
    .await
}

pub async fn get_all(db: &sqlx::PgPool) -> Result<Vec<ShortModel>, sqlx::Error> {
    sqlx::query_as::<_, ShortModel>(
        "SELECT id, long_url, key, description, views, created_at FROM short ORDER BY created_at DESC LIMIT 1000"
    )
    .fetch_all(db)
    .await
}

pub async fn increase_views(db: &sqlx::PgPool, key: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE short SET views = views + 1 WHERE key = $1")
        .bind(key)
        .execute(db)
        .await?;

    Ok(())
}

pub async fn delete(db: &sqlx::PgPool, key: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM short WHERE key = $1")
        .bind(key)
        .execute(db)
        .await?;

    Ok(())
}
