use crate::error::HtmlError;
use askama::Template;
use axum::response::Html;

pub fn render<T: Template>(template: T) -> Result<Html<String>, HtmlError> {
    Ok(Html(template.render().unwrap()))
}
