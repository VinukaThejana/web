#[derive(Default, Clone)]
pub struct AppState {}

impl AppState {
    pub async fn new() -> Self {
        Self {}
    }
}

impl AppState {
    pub async fn close(&self) {}
}
