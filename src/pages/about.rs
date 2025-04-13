use crate::{
    cache::project::{gck_projects, gct_projects},
    config::state::AppState,
    database,
    error::AppError,
    model::project::{Project, ToProjects},
    util::{Cache, from_cache, to_cache},
};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use redis::RedisResult;

#[derive(Debug, Template, Default)]
#[template(path = "about.html")]
pub struct About {
    pub projects: Vec<Project>,
}

pub async fn render(State(state): State<AppState>) -> impl IntoResponse {
    let ck = gck_projects();
    let ct = gct_projects();

    let mut about = About::default();

    let conn = state.get_redis_conn().await.map_err(AppError::Other);
    if conn.is_err() {
        return Html(about.render().unwrap());
    }
    let mut conn = conn.unwrap();

    let payload: Option<String> = redis::cmd("GET")
        .arg(&ck)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))
        .unwrap_or(None);
    if payload.is_some() {
        Cache::HIT.log(&ck);
        about.projects = from_cache(&payload.unwrap());
        return Html(about.render().unwrap());
    }

    Cache::MISS.log(&ck);

    let projects = database::project::get(&state.db)
        .await
        .unwrap_or(vec![])
        .to_projects();
    let payload = to_cache(&projects);

    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(&ck)
            .arg(payload)
            .arg("EX")
            .arg(ct)
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&ck);
            return;
        }
        Cache::SET.log(&ck);
    });

    about.projects = projects;
    Html(about.render().unwrap())
}
