use crate::{
    config::{ENV, state::AppState},
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
use std::time::Duration;

#[derive(Debug, Template, Default)]
#[template(path = "about.html")]
pub struct About {
    pub projects: Vec<Project>,
}

const CACHE_PATH: &str = "about-projects";
pub fn get_cache_key() -> String {
    format!("{}:projects", &ENV.redis_schema)
}

pub async fn render(State(state): State<AppState>) -> impl IntoResponse {
    let mut about = About::default();

    let conn = state.get_redis_conn().await.map_err(AppError::Other);
    if conn.is_err() {
        return Html(about.render().unwrap());
    }
    let mut conn = conn.unwrap();

    let payload: Option<String> = redis::cmd("GET")
        .arg(get_cache_key())
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))
        .unwrap_or(None);
    if payload.is_some() {
        Cache::HIT.log(CACHE_PATH);
        about.projects = from_cache(&payload.unwrap());
        return Html(about.render().unwrap());
    }

    Cache::MISS.log(CACHE_PATH);

    let projects = database::project::get(&state.db)
        .await
        .unwrap_or(vec![])
        .to_projects();
    let payload = to_cache(&projects);

    tokio::spawn(async move {
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(get_cache_key())
            .arg(payload)
            .arg("EX")
            .arg(Duration::from_secs(90 * 24 * 60 * 60).as_secs())
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(CACHE_PATH);
            return;
        }
        Cache::SET.log(CACHE_PATH);
    });

    about.projects = projects;
    Html(about.render().unwrap())
}
