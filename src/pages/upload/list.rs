use crate::{
    config::{ENV, state::AppState},
    error::{AppError, HtmlError},
    model::aws_object::AwsObject,
};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};

#[derive(Default, Template)]
#[template(path = "upload/list.html")]
pub struct Tmpl<'a> {
    pub active: &'a str,
    pub objects: Vec<AwsObject>,
}

impl Tmpl<'_> {
    pub fn new(objects: Vec<AwsObject>) -> Self {
        Self {
            active: "list",
            objects,
        }
    }
}

pub async fn render(State(state): State<AppState>) -> Result<impl IntoResponse, HtmlError> {
    let objects = state
        .s3
        .list_objects_v2()
        .bucket(&*ENV.cloudflare_bucket_name)
        .send()
        .await
        .map_err(AppError::from_generic_error)?;

    let objects: Vec<AwsObject> = objects
        .contents()
        .iter()
        .map(|object| object.into())
        .collect();

    Ok(Html(
        Tmpl::new(objects)
            .render()
            .map_err(AppError::from_generic_error)?,
    ))
}
