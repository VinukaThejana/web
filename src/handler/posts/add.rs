use super::{CaptchaFailed, Failed, Invalid, Okay};
use crate::{
    cache::{
        self,
        post::{gck_for_home, gck_for_page, gck_for_slug, gck_for_slugs},
    },
    config::{ENV, state::AppState},
    database,
    error::{AppError, HtmlError},
    model::post::AddPost,
    util::{
        POST_LIMIT, ClientIp, cloudflare_verify,
        html::{self},
    },
};
use axum::{
    Form,
    extract::State,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use redis::RedisResult;
use validator::Validate;

const FORM_ID: &str = "create-post-form";

pub async fn run(
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Form(payload): Form<AddPost>,
) -> Result<impl IntoResponse, HtmlError> {
    if !cloudflare_verify(state.http(), &payload.cf_turnstile_response, &ip).await {
        return html::render(CaptchaFailed::new(FORM_ID));
    }

    if let Err(e) = payload.validate() {
        return html::render(Invalid::new(FORM_ID, &AppError::Validation(e).to_string()));
    }

    if payload.password != (*ENV.admin_password) {
        return html::render(Invalid::new(FORM_ID, "password is incorrect"));
    }

    let mut conn = state.redis().await?;
    if let Err(e) = database::post::add(state.db().await, &payload)
        .await
        .map_err(AppError::from_database_error)
    {
        match e {
            AppError::UniqueViolation { .. } => {
                return html::render(Invalid::new(FORM_ID, "post with this slug already exists"));
            }
            _ => {
                log::error!("failed to add post: {}", e);
                return html::render(Failed::default());
            }
        }
    }
    log::info!("post addded");

    let tp = cache::post::gtp(state.clone(), true).await;
    if let Err(e) = tp {
        log::error!("failed to get total posts: {}", e);
        let result: RedisResult<()> = redis::pipe()
            .flushdb()
            .ignore()
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            log::error!("failed to flush redis: {}", e);
        }

        return html::render(Okay::new(
            "Added",
            "Your post has been successfully created. It will appear in the blog soon!",
        ));
    }
    let tp = tp.unwrap();
    let tp = (tp as f64 / POST_LIMIT as f64).ceil() as u64;

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

    html::render(Okay::new(
        "Added",
        "Your post has been successfully created. It will appear in the blog soon!",
    ))
}
