use crate::{
    cache::{
        experience::{gck_experiences, gct_experiences},
        project::{gck_projects, gct_projects},
    },
    config::state::AppState,
    database,
    error::{AppError, HtmlError},
    model::{
        experience::{Experience, ToExperiences},
        project::{Project, ToProjects},
    },
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
    pub experiences: Vec<Experience>,
}

pub async fn render(State(state): State<AppState>) -> Result<impl IntoResponse, HtmlError> {
    let mut about = Tmpl::default();
    let mut conn = state.redis().await?;

    // --- Projects Cache/Fetch ---
    let ck_proj = gck_projects();
    let ct_proj = gct_projects();
    let payload_proj: Option<String> = redis::cmd("GET")
        .arg(&ck_proj)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))
        .unwrap_or(None);

    let projects = if let Some(p) = payload_proj {
        Cache::HIT.log(&ck_proj);
        from_cache(&p)
    } else {
        Cache::MISS.log(&ck_proj);
        let projs = database::project::get(state.db().await)
            .await
            .unwrap_or(vec![])
            .to_projects();
        let payload = to_cache(&projs);
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(&ck_proj)
            .arg(payload)
            .arg("EX")
            .arg(ct_proj)
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&ck_proj);
        } else {
            Cache::SET.log(&ck_proj);
        }
        projs
    };
    about.projects = projects;

    // --- Experiences Cache/Fetch ---
    let ck_exp = gck_experiences();
    let ct_exp = gct_experiences();
    let payload_exp: Option<String> = redis::cmd("GET")
        .arg(&ck_exp)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Other(e.into()))
        .unwrap_or(None);

    let experiences = if let Some(p) = payload_exp {
        Cache::HIT.log(&ck_exp);
        from_cache(&p)
    } else {
        Cache::MISS.log(&ck_exp);
        let exps = database::experience::get(state.db().await)
            .await
            .unwrap_or(vec![])
            .to_experiences();
        let payload = to_cache(&exps);
        let result: RedisResult<()> = redis::cmd("SET")
            .arg(&ck_exp)
            .arg(payload)
            .arg("EX")
            .arg(ct_exp)
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("{:?}", e);
            Cache::FAILED.log(&ck_exp);
        } else {
            Cache::SET.log(&ck_exp);
        }
        exps
    };
    about.experiences = experiences;

    Ok(Html(about.render().unwrap()))
}
