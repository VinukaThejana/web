pub mod home;

use std::net::SocketAddr;

use crate::{
    cache::{
        self,
        post::{gck_for_home, gck_for_page, gck_for_slug, gck_for_slugs},
    },
    config::{ENV, state::AppState},
    database,
    error::AppError,
    model::post::AddPost,
    util::{POST_LIMIT, cloudflare_verify},
};
use askama::Template;
use axum::{
    Form,
    extract::{ConnectInfo, State},
    response::{Html, IntoResponse},
};
use chrono::{DateTime, Utc};
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

#[derive(Debug, Default, Template)]
#[template(path = "components/add-post/captcha.html")]
pub struct PostCaptchaFailed {}

pub async fn add(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<AddPost>,
) -> Result<impl IntoResponse, AppError> {
    let ip = addr.ip().to_string();
    if !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return Ok(Html(PostCaptchaFailed::default().render().unwrap()));
    }

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

    Ok(Html(PostAddSuccess::default().render().unwrap()))
}
