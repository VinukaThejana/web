pub mod home;

use crate::{
    cache::{
        self,
        post::{gck_for_home, gck_for_page, gck_for_slug, gck_for_slugs},
    },
    config::{ENV, state::AppState},
    database,
    error::{AppError, HtmlError},
    model::post::{AddPost, DelPost},
    util::{
        POST_LIMIT, cloudflare_verify,
        html::{self},
    },
};
use askama::Template;
use axum::{
    Form,
    extract::{ConnectInfo, State},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use redis::RedisResult;
use std::net::SocketAddr;
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

#[derive(Debug, Default, Template)]
#[template(path = "components/add-post/captcha.html")]
pub struct PostCaptchaFailed {}

pub async fn add(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<AddPost>,
) -> Result<impl IntoResponse, HtmlError> {
    let ip = addr.ip().to_string();

    if !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return html::render(PostCaptchaFailed::default());
    }

    if let Err(e) = payload.validate() {
        return html::render(PostAddInvaid::new(&AppError::Validation(e).to_string()));
    }

    if payload.password != (*ENV.admin_password) {
        return html::render(PostAddInvaid::new("password is incorrect"));
    }

    let conn = state.get_redis_conn().await.map_err(AppError::Other);
    if let Err(e) = conn {
        log::error!("failed to get redis connection: {}", e);
        return html::render(PostAddFailed::default());
    }
    let mut conn = conn.unwrap();

    if let Err(e) = database::post::add(&state.db, &payload)
        .await
        .map_err(AppError::from_database_error)
    {
        if let AppError::UniqueViolation(_) = e {
            return html::render(PostAddInvaid::new("post with this slug already exists"));
        }

        log::error!("failed to add post: {}", e);
        return html::render(PostAddFailed::default());
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

        return html::render(PostAddSuccess::default());
    }
    let tp = tp.unwrap();
    let tp = (tp as f64 / POST_LIMIT as f64).ceil() as u64;

    tokio::spawn(async move {
        if let Err(e) = cache::post::update_last_modified(
            state.clone(),
            tp,
            &DateTime::<Utc>::from_timestamp(payload.date.try_into().unwrap(), 0)
                .unwrap()
                .to_rfc3339(),
        )
        .await
        {
            log::error!("failed to update last modified: {}", e);
        }

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
            .cmd("DEL")
            .arg(gck_for_slugs())
            .ignore()
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("failed to delete redis cache: {}", e);
        }
    });

    html::render(PostAddSuccess::default())
}

#[derive(Debug, Default, Template)]
#[template(path = "components/del-post/captcha.html")]
pub struct DelCaptchaFailed {}

#[derive(Debug, Default, Template)]
#[template(path = "components/del-post/invalid.html")]
pub struct DelInvalid<'a> {
    pub message: &'a str,
}
impl<'a> DelInvalid<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }
}

#[derive(Debug, Default, Template)]
#[template(path = "components/del-post/success.html")]
pub struct DelOkay {}

#[derive(Debug, Default, Template)]
#[template(path = "components/del-post/failed.html")]
pub struct DelFailed {}

pub async fn delete(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<DelPost>,
) -> Result<impl IntoResponse, HtmlError> {
    let ip = addr.ip().to_string();

    if !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return html::render(DelCaptchaFailed::default());
    }

    if let Err(e) = payload.validate() {
        return html::render(DelInvalid::new(&AppError::Validation(e).to_string()));
    }

    if payload.password != (*ENV.admin_password) {
        return html::render(DelInvalid::new("password is incorrect"));
    }

    let conn = state.get_redis_conn().await.map_err(AppError::Other);
    if let Err(e) = conn {
        log::error!("failed to get redis connection: {}", e);
        return html::render(DelFailed::default());
    }
    let mut conn = conn.unwrap();

    if let Err(e) = database::post::del_by_slug(&state.db, &payload.slug)
        .await
        .map_err(AppError::from_database_error)
    {
        log::error!("failed to delete post: {}", e);
        return html::render(DelFailed::default());
    }

    let result: RedisResult<()> = redis::pipe()
        .flushdb()
        .ignore()
        .query_async(&mut conn)
        .await;
    if let Err(err) = result {
        log::error!("failed to flush the redis database: {}", err);
        return html::render(DelFailed::default());
    }

    html::render(DelOkay::default())
}
