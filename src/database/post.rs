use crate::{
    model::post::{AddPost, PartialPost, PartialPostWithSlug, PostModel},
    util,
};

pub async fn get(db: &sqlx::PgPool) -> Result<Vec<PartialPost>, sqlx::Error> {
    sqlx::query_as::<_, PartialPost>(
        "SELECT id, title, slug, summary, tags, date FROM post ORDER BY id DESC LIMIT $1",
    )
    .bind(util::POST_LIMIT as i64)
    .fetch_all(db)
    .await
}

pub async fn get_slugs(db: &sqlx::PgPool) -> Result<Vec<PartialPostWithSlug>, sqlx::Error> {
    sqlx::query_as::<_, PartialPostWithSlug>("SELECT id, date, slug FROM post ORDER BY id ASC")
        .fetch_all(db)
        .await
}

#[derive(PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

pub async fn get_by_page(
    db: &sqlx::PgPool,
    page: u64,
    order: Order,
    has_initial_data: bool,
) -> Result<Vec<PartialPost>, sqlx::Error> {
    let limit = util::POST_LIMIT as i64;
    let offset = (if has_initial_data { limit } else { 0 }) + ((page as i64 - 1) * limit);
    let limit_plus_one = limit + 1;

    if order == Order::Desc {
        sqlx::query_as::<_, PartialPost>(
            "SELECT id, title, slug, summary, tags, date FROM post ORDER BY id DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit_plus_one)
        .bind(offset)
        .fetch_all(db)
        .await
    } else {
        sqlx::query_as::<_, PartialPost>(
            "SELECT id, title, slug, summary, tags, date FROM post ORDER BY id ASC LIMIT $1 OFFSET $2"
        )
        .bind(limit_plus_one)
        .bind(offset)
        .fetch_all(db)
        .await
    }
}

pub async fn get_by_slug(db: &sqlx::PgPool, slug: &str) -> Result<PostModel, sqlx::Error> {
    sqlx::query_as::<_, PostModel>(
        "SELECT id, title, seo_title, slug, photo_url, tags, summary, content, date FROM post WHERE slug = $1"
    )
    .bind(slug.to_lowercase())
    .fetch_one(db)
    .await
}

pub async fn get_by_id(db: &sqlx::PgPool, id: i32) -> Result<PostModel, sqlx::Error> {
    sqlx::query_as::<_, PostModel>(
        "SELECT id, title, seo_title, slug, photo_url, tags, summary, content, date FROM post WHERE id = $1"
    )
    .bind(id)
    .fetch_one(db)
    .await
}

pub async fn update(db: &sqlx::PgPool, post: &PostModel) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE post SET title = $1, seo_title = $2, slug = $3, summary = $4, photo_url = $5, tags = $6, content = $7 WHERE id = $8"
    )
    .bind(&post.title)
    .bind(&post.seo_title)
    .bind(&post.slug)
    .bind(&post.summary)
    .bind(&post.photo_url)
    .bind(&post.tags)
    .bind(&post.content)
    .bind(post.id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_total_posts(db: &sqlx::PgPool) -> Result<u64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM post")
        .fetch_one(db)
        .await?;
    Ok(count.0 as u64)
}

pub async fn add(db: &sqlx::PgPool, payload: &AddPost) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO post (title, seo_title, slug, summary, photo_url, tags, content, date) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&payload.title)
    .bind(&payload.seo_title)
    .bind(&payload.slug)
    .bind(&payload.summary)
    .bind(&payload.photo_url)
    .bind(&payload.tags)
    .bind(&payload.content)
    .bind(payload.date as i32)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn del_by_slug(db: &sqlx::PgPool, slug: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM post WHERE slug = $1")
        .bind(slug.to_lowercase())
        .execute(db)
        .await?;

    Ok(())
}
