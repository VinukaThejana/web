use crate::{
    config::{ENV, state::AppState},
    database,
    error::{AppError, HtmlError},
    model::post::EditPost,
    util::{
        ClientIp, cloudflare_verify,
        html::{self},
    },
};
use axum::{
    Form,
    extract::State,
    response::IntoResponse,
};
use redis::RedisResult;
use validator::Validate;

use super::{CaptchaFailed, Failed, Invalid, Okay};

const FORM_ID: &str = "edit-post-form";

pub async fn run(
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Form(payload): Form<EditPost>,
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
    if let Err(e) = database::post::update(
        state.db().await,
        &crate::model::post::PostModel {
            id: payload.id,
            title: payload.title,
            seo_title: payload.seo_title,
            slug: payload.slug,
            photo_url: payload.photo_url,
            content: payload.content,
            tags: payload.tags,
            summary: payload.summary,
            date: payload.date.try_into().unwrap(),
        },
    )
    .await
    .map_err(AppError::from_database_error)
    {
        match e {
            AppError::UniqueViolation { .. } => {
                return html::render(Invalid::new(FORM_ID, "post with this slug already exists"));
            }
            _ => {
                log::error!("failed to update post: {}", e);
                return html::render(Failed::default());
            }
        }
    };

    let result: RedisResult<()> = redis::pipe()
        .flushdb()
        .ignore()
        .query_async(&mut conn)
        .await;
    if let Err(e) = result {
        log::error!("failed to flush redis: {}", e);
        return html::render(Failed::default());
    }

    html::render(Okay::new(
        "Edited",
        "Your post has been successfully edited. It will be visible to others shortly.",
    ))
}
