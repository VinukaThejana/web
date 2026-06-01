use crate::model::experience::ExperienceModel;

pub async fn get(db: &sqlx::PgPool) -> Result<Vec<ExperienceModel>, sqlx::Error> {
    sqlx::query_as::<_, ExperienceModel>(
        "SELECT id, title, company, description, tags, date FROM experience ORDER BY id DESC"
    )
    .fetch_all(db)
    .await
}
