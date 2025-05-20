use crate::{error::HtmlError, util::html};
use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "upload/delete.html")]
pub struct Tmpl {
    pub active: &'static str,
}

impl Default for Tmpl {
    fn default() -> Self {
        Self { active: "delete" }
    }
}

pub async fn render() -> Result<impl IntoResponse, HtmlError> {
    html::render(Tmpl::default())
}
