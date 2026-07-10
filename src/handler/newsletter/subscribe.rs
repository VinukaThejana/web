use crate::{
    config::{ENV, state::AppState},
    error::{AppError, HtmlError},
    model::newsletter::SignUp,
    util::{ClientIp, cloudflare_verify, html},
};
use askama::Template;
use axum::{Form, extract::State, response::IntoResponse};
use resend_rs::types::ContactData;
use validator::Validate;

#[derive(Debug, Default, Template)]
#[template(path = "components/newsletter/failed.html")]
struct Failed {}

#[derive(Debug, Default, Template)]
#[template(path = "components/newsletter/success.html")]
struct Okay {}

#[derive(Debug, Default, Template)]
#[template(path = "components/newsletter/invalid.html")]
struct Invalid {}

#[derive(Debug, Default, Template)]
#[template(path = "components/newsletter/captcha.html")]
struct CaptchaFailed {}

pub async fn run(
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Form(payload): Form<SignUp>,
) -> Result<impl IntoResponse, HtmlError> {
    if !cloudflare_verify(state.http(), &payload.cf_turnstile_response, &ip).await {
        return html::render(CaptchaFailed::default());
    }

    if let Err(e) = payload.validate() {
        log::info!("validation error: {:?}", AppError::Validation(e));
        return html::render(Invalid::default());
    }

    let contact = ContactData::new(&payload.email).with_unsubscribed(false);
    if let Err(e) = state
        .resend()
        .contacts
        .create(&ENV.resend_audience_id, contact)
        .await
    {
        log::error!("error creating contact: {:?}", e);
        return html::render(Failed::default());
    }

    html::render(Okay::default())
}
