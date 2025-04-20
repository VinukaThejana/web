use crate::{
    config::{ENV, state::AppState},
    error::{AppError, HtmlError},
    model::aws_object::AwsObject,
};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};
use serde::Deserialize;

const LIMIT: i32 = 10;

#[derive(Default, Template)]
#[template(path = "upload/list.html")]
pub struct Tmpl<'a> {
    pub active: &'a str,
    pub next_key: &'a str,
    pub objects: Vec<AwsObject>,
    pub has_next: bool,
}

impl<'a> Tmpl<'a> {
    pub fn new(objects: Vec<AwsObject>, next_key: &'a str, has_next: bool) -> Self {
        Self {
            active: "list",
            objects,
            next_key,
            has_next,
        }
    }
}

#[derive(Deserialize)]
pub struct Params {
    pub key: Option<String>,
}

pub async fn render(
    State(state): State<AppState>,
    query: Query<Params>,
) -> Result<impl IntoResponse, HtmlError> {
    let key = query
        .key
        .as_ref()
        .map(|k| {
            String::from_utf8(base64_url::decode(k).ok().unwrap_or_default()).unwrap_or_default()
        })
        .unwrap_or_default();

    let objects = state
        .s3
        .list_objects_v2()
        .bucket(&*ENV.cloudflare_bucket_name)
        .start_after(&key)
        .max_keys(LIMIT + 1)
        .send()
        .await
        .map_err(AppError::from_generic_error)?;

    let objects: Vec<AwsObject> = objects
        .contents()
        .iter()
        .map(|object| object.into())
        .collect();

    let has_next = objects.len() > LIMIT as usize;
    let objects = if has_next {
        objects.into_iter().take(LIMIT as usize).collect()
    } else {
        objects
    };
    let next_key = base64_url::encode(
        &objects
            .last()
            .map(|object| object.path.clone())
            .unwrap_or_default(),
    );

    Ok(Html(
        Tmpl::new(objects, &next_key, has_next)
            .render()
            .map_err(AppError::from_generic_error)?,
    ))
}
