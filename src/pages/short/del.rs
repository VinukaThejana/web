use askama::Template;
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    key: Option<String>,
}

#[derive(Debug, Template)]
#[template(path = "short/del.html")]
pub struct Tmpl {
    pub active: &'static str,
    pub key: Option<String>,
}

impl Tmpl {
    pub fn new(key: Option<String>) -> Self {
        Self { active: "del", key }
    }
}

pub async fn render(Query(query): Query<DeleteQuery>) -> impl IntoResponse {
    Html(Tmpl::new(query.key).render().unwrap())
}
