use crate::model::project::ProjectModel;

pub async fn get(db: &sqlx::PgPool) -> Result<Vec<ProjectModel>, sqlx::Error> {
    sqlx::query_as::<_, ProjectModel>(
        "SELECT id, title, description, tags, url, date FROM project ORDER BY id DESC"
    )
    .fetch_all(db)
    .await
}
