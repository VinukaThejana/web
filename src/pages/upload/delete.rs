use crate::{error::HtmlError, util::html};
use askama::Template;
use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeleteQuery {
    key: Option<String>,
}

#[derive(Template)]
#[template(path = "upload/delete.html")]
pub struct Tmpl {
    pub active: &'static str,
    pub key: Option<String>,
}

impl Tmpl {
    pub fn new(key: Option<String>) -> Self {
        Self {
            active: "delete",
            key,
        }
    }
}

pub async fn render(Query(query): Query<DeleteQuery>) -> Result<impl IntoResponse, HtmlError> {
    html::render(Tmpl::new(query.key))
}
