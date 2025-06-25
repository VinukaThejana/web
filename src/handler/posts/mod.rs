use askama::Template;

pub mod add;
pub mod delete;
pub mod edit;
pub mod load_more;

#[derive(Debug, Template)]
#[template(path = "components/posts/success.html")]
pub(crate) struct Okay<'a> {
    pub action: &'a str,
    pub message: &'a str,
}

impl<'a> Okay<'a> {
    pub fn new(action: &'a str, message: &'a str) -> Self {
        Self { action, message }
    }
}

#[derive(Debug, Template)]
#[template(path = "components/posts/invalid.html")]
pub(crate) struct Invalid<'a> {
    form_id: &'a str,
    message: &'a str,
}
impl<'a> Invalid<'a> {
    pub fn new(form_id: &'a str, message: &'a str) -> Self {
        Self { form_id, message }
    }
}

#[derive(Debug, Template)]
#[template(path = "components/posts/captcha.html")]
pub(crate) struct CaptchaFailed<'a> {
    pub form_id: &'a str,
}

impl<'a> CaptchaFailed<'a> {
    pub fn new(form_id: &'a str) -> Self {
        Self { form_id }
    }
}

#[derive(Debug, Default, Template)]
#[template(path = "components/posts/failed.html")]
pub(crate) struct Failed {}
