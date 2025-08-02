use askama::Template;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::{
    config::state::AppState,
    database,
    error::{AppError, HtmlError},
    util::html,
};

#[derive(Debug, Template)]
#[template(path = "posts/edit.html")]
pub struct Tmpl<'a> {
    pub id: i32,
    pub date: i32,
    pub title: &'a str,
    pub seo_title: &'a str,
    pub slug: &'a str,
    pub summary: &'a str,
    pub photo_url: &'a str,
    pub tags: &'a str,
    pub content: &'a str,
}

impl<'a> Tmpl<'a> {
    pub fn new(post: &'a entity::post::Model) -> Self {
        Self {
            id: post.id,
            date: post.date,
            title: &post.title,
            seo_title: &post.seo_title,
            slug: &post.slug,
            summary: &post.summary,
            photo_url: &post.photo_url,
            tags: &post.tags,
            content: &post.content,
        }
    }
}

pub async fn render(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, HtmlError> {
    let post = database::post::get_by_slug(&state.db, &slug)
        .await
        .map_err(AppError::from_database_error)?;

    html::render(Tmpl::new(&post))
}
