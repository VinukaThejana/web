use crate::{
    config::state::AppState,
    database,
    error::{AppError, HtmlError},
    model::short::{Link, ToLinks},
};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};

#[derive(Default, Template)]
#[template(path = "short/list.html")]
pub struct Tmpl {
    pub links: Vec<Link>,
}
impl Tmpl {
    pub fn new(links: Vec<Link>) -> Self {
        Self { links }
    }
}

pub async fn render(State(state): State<AppState>) -> Result<impl IntoResponse, HtmlError> {
    let links = database::short::get_all(&state.db)
        .await
        .map_err(AppError::from_database_error)?
        .to_links();

    Ok(Html(Tmpl::new(links).render().unwrap()))
}
