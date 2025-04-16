use std::net::SocketAddr;

use crate::{
    config::state::AppState,
    error::HtmlError,
    model::contact::ContactUs,
    util::{AUTHOR_EMAIL, cloudflare_verify, srilankan_time},
};
use askama::Template;
use axum::{
    Form,
    extract::{ConnectInfo, State},
    response::{Html, IntoResponse},
};
use resend_rs::types::CreateEmailBaseOptions;
use validator::Validate;

#[derive(Debug, Default, Template)]
#[template(path = "components/contact/success.html")]
pub struct Success {}

#[derive(Debug, Default, Template)]
#[template(path = "components/contact/failed.html")]
pub struct Failed {}

#[derive(Debug, Default, Template)]
#[template(path = "components/contact/invalid.html")]
pub struct Invalid<'a> {
    message: &'a str,
}
impl<'a> Invalid<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }
}

#[derive(Debug, Default, Template)]
#[template(path = "components/contact/captcha.html")]
pub struct CaptchaFailed {}

#[derive(Debug, Default, Template)]
#[template(path = "components/email/contact.html")]
pub struct SendEmail<'a> {
    name: &'a str,
    email: &'a str,
    message: &'a str,
    date: &'a str,
}
impl<'a> SendEmail<'a> {
    pub fn new(name: &'a str, email: &'a str, message: &'a str, date: &'a str) -> Self {
        Self {
            name,
            email,
            message,
            date,
        }
    }
}

pub async fn send_msg(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Form(payload): Form<ContactUs>,
) -> Result<impl IntoResponse, HtmlError> {
    let ip = addr.ip().to_string();
    if !cloudflare_verify(&payload.cf_turnstile_response, &ip).await {
        return Ok(Html(CaptchaFailed::default().render().unwrap()));
    }

    if let Err(e) = payload.validate() {
        let message = e
            .field_errors()
            .values()
            .flat_map(|e| e.iter())
            .filter_map(|err| {
                err.message
                    .as_ref()
                    .map(|msg| msg.to_string())
                    .or(Some(String::from("invalid value")))
            })
            .next()
            .unwrap_or(String::from("invalid value"));

        return Ok(Html(Invalid::new(&message).render().unwrap()));
    }

    let email = CreateEmailBaseOptions::new(
        "vinuka.dev <contact-form@vinuka.dev>",
        [AUTHOR_EMAIL],
        format!("[{}] Contact form submission", &payload.name),
    )
    .with_html(
        &SendEmail::new(
            &payload.name,
            &payload.email,
            &payload.message,
            &srilankan_time().format("%Y-%m-%d %H:%M:%S").to_string(),
        )
        .render()
        .unwrap(),
    );
    let result = state.rs.emails.send(email).await;
    if let Err(e) = result {
        log::error!("failed to send email : {:?}", e);
        return Ok(Html(Failed::default().render().unwrap()));
    }

    Ok(Html(Success::default().render().unwrap()))
}
