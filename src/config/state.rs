use resend_rs::Resend;

use super::ENV;

#[derive(Default, Clone)]
pub struct AppState {
    pub rs: Resend,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            rs: Resend::new(&ENV.resend_api_key),
        }
    }
}

impl AppState {
    pub async fn close(&self) {}
}
