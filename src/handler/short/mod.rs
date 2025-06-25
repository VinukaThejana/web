use askama::Template;

pub mod add;
pub mod delete;
pub mod verify;

#[derive(Debug, Template)]
#[template(path = "components/short/captcha.html")]
pub(crate) struct CaptchaFailed<'a> {
    pub form_id: &'a str,
}

impl<'a> CaptchaFailed<'a> {
    pub fn new(form_id: &'a str) -> Self {
        Self { form_id }
    }
}

#[derive(Debug, Template)]
#[template(path = "components/short/invalid.html")]
pub(crate) struct Invalid<'a> {
    pub form_id: &'a str,
    pub message: &'a str,
}
impl<'a> Invalid<'a> {
    pub fn new(form_id: &'a str, message: &'a str) -> Self {
        Self { form_id, message }
    }
}

#[derive(Debug, Default, Template)]
#[template(path = "components/short/failed.html")]
pub(crate) struct Failed {}

#[derive(Debug, Template)]
#[template(path = "components/short/success.html")]
pub(crate) struct Okay<'a> {
    action: &'a str,
    message: &'a str,
}

impl<'a> Okay<'a> {
    pub fn new(action: &'a str, message: &'a str) -> Self {
        Self { action, message }
    }
}
