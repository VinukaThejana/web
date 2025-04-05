use crate::{
    config::{ENV, state::AppState},
    error::AppError,
    model::newsletter::SignUp,
};
use askama::Template;
use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse},
};
use resend_rs::types::ContactData;
use validator::Validate;

pub async fn subscribe(
    State(state): State<AppState>,
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

    let contact = ContactData::new(&payload.email).with_unsubscribed(false);
    if let Err(e) = state
        .rs
        .contacts
        .create(&ENV.resend_audience_id, contact)
        .await
    {
        log::error!("error creating contact: {:?}", e);
        return Ok(Html(Failed::default().render().unwrap()));
    }

    Ok(Html(Okay::default().render().unwrap()))
}
