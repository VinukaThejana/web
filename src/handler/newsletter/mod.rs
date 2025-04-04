use crate::{config::state::AppState, error::AppError, model::newsletter::SignUp};
use askama::Template;
use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse},
};
use validator::Validate;

pub async fn subscribe(
    State(_state): State<AppState>,
    Form(payload): Form<SignUp>,
) -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Default, Template)]
    #[template(path = "components/newsletter/status-failed.html")]
    struct Failed {}

    #[derive(Debug, Default, Template)]
    #[template(path = "components/newsletter/status-success.html")]
    struct Okay {}

    #[derive(Debug, Default, Template)]
    #[template(path = "components/newsletter/status-email-invalid.html")]
    struct Invalid {}

    if let Err(e) = payload.validate() {
        log::info!("validation error: {:?}", AppError::Validation(e));
        return Ok(Html(Invalid::default().render().unwrap()));
    }

    Ok(Html(Okay::default().render().unwrap()))
}
