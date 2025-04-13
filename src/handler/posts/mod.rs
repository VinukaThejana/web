pub mod home;

use crate::{
    cache::{
        self,
        post::{gck_for_home, gck_for_page, gck_for_slug},
    },
    config::{ENV, state::AppState},
    database,
    error::AppError,
    model::post::AddPost,
    util::POST_LIMIT,
};
use askama::Template;
use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse},
};
use redis::RedisResult;
use validator::Validate;

#[derive(Debug, Default, Template)]
#[template(path = "components/add-post/success.html")]
pub struct PostAddSuccess {}

#[derive(Debug, Default, Template)]
#[template(path = "components/add-post/failed.html")]
pub struct PostAddFailed {}

#[derive(Debug, Default, Template)]
#[template(path = "components/add-post/invalid.html")]
pub struct PostAddInvaid<'a> {
    message: &'a str,
}
impl<'a> PostAddInvaid<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }
}

pub async fn add(
    State(state): State<AppState>,
    Form(payload): Form<AddPost>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(e) = payload.validate() {
        let message = e
            .field_errors()
            .values()
            .flat_map(|e| e.iter())
            .filter_map(|err| {
                err.message
                    .as_ref()
                    .map(|msg| msg.to_string())
                    .or(Some(String::from("invalid value")))
            })
            .next()
            .unwrap_or(String::from("invalid value"));

        return Ok(Html(PostAddInvaid::new(&message).render().unwrap()));
    }

    if payload.password != (*ENV.admin_password) {
        return Ok(Html(
            PostAddInvaid::new("password is incorrect")
                .render()
                .unwrap(),
        ));
    }

    let conn = state.get_redis_conn().await.map_err(AppError::Other);
    if let Err(e) = conn {
        log::error!("failed to get redis connection: {}", e);
        return Ok(Html(PostAddFailed::default().render().unwrap()));
    }
    let mut conn = conn.unwrap();

    if let Err(e) = database::post::add(&state.db, &payload)
        .await
        .map_err(AppError::from_database_error)
    {
        if let AppError::UniqueViolation(_) = e {
            return Ok(Html(
                PostAddInvaid::new("post with this slug already exists")
                    .render()
                    .unwrap(),
            ));
        }

        log::error!("failed to add post: {}", e);
        return Ok(Html(PostAddFailed::default().render().unwrap()));
    }
    log::info!("post addded");

    let tp = cache::post::gtp(state.clone(), true).await;
    if let Err(e) = tp {
        log::error!("failed to get total posts: {}", e);
        tokio::spawn(async move {
            let result: RedisResult<()> = redis::pipe()
                .flushdb()
                .ignore()
                .query_async(&mut conn)
                .await;
            if let Err(e) = result {
                log::error!("failed to flush redis: {}", e);
            }
        });

        return Ok(Html(PostAddFailed::default().render().unwrap()));
    }
    let tp = tp.unwrap();
    let tp = (tp as f64 / POST_LIMIT as f64).ceil() as u64;

    tokio::spawn(async move {
        let result: RedisResult<()> = redis::pipe()
            .cmd("DEL")
            .arg(gck_for_home())
            .ignore()
            .cmd("DEL")
            .arg(gck_for_slug(&payload.slug))
            .ignore()
            .cmd("DEL")
            .arg(gck_for_page(tp))
            .ignore()
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("failed to delete redis cache: {}", e);
        }
    });

    Ok(Html(PostAddSuccess::default().render().unwrap()))
}
