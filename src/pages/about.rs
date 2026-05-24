use crate::{
    cache::project::{gck_projects, gct_projects},
    config::state::AppState,
    database,
    error::{AppError, HtmlError},
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
pub struct Tmpl {
    pub projects: Vec<Project>,
}

pub async fn render(State(state): State<AppState>) -> Result<impl IntoResponse, HtmlError> {
    let ck = gck_projects();
    let ct = gct_projects();

    let mut about = Tmpl::default();

    let mut conn = state.redis().await?;

    let payload: Option<String> = redis::cmd("GET")
        .arg(&ck)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))
        .unwrap_or(None);
    if let Some(payload) = payload {
        Cache::HIT.log(&ck);
        about.projects = from_cache(&payload);
        return Ok(Html(about.render().unwrap()));
    }

    Cache::MISS.log(&ck);

    let projects = database::project::get(state.db().await)
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
    Ok(Html(about.render().unwrap()))
}
