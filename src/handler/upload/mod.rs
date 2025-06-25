pub mod cdn;
pub mod delete;
pub mod presigned;

use askama::Template;

#[derive(Debug, Template)]
#[template(path = "components/upload/captcha.html")]
pub(crate) struct CaptchaFailed<'a> {
    pub form_id: &'a str,
}

impl<'a> CaptchaFailed<'a> {
    pub fn new(form_id: &'a str) -> Self {
        Self { form_id }
    }
}

#[derive(Debug, Default, Template)]
#[template(path = "components/upload/failed.html")]
pub struct Failed {}

#[derive(Debug, Template)]
#[template(path = "components/upload/invalid.html")]
pub(crate) struct Invalid<'a> {
    pub form_id: &'a str,
    pub message: &'a str,
}
impl<'a> Invalid<'a> {
    pub fn new(form_id: &'a str, message: &'a str) -> Self {
        Self { form_id, message }
    }
}

#[derive(Debug, Template)]
#[template(path = "components/upload/success.html")]
pub(crate) struct Okay<'a> {
    pub action: &'a str,
    pub message: &'a str,
}

impl<'a> Okay<'a> {
    pub fn new(action: &'a str, message: &'a str) -> Self {
        Self { action, message }
    }
}
