use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct Payload {
    #[validate(length(min = 1, message = "path cannot be empty"))]
    pub path: String,

    #[validate(length(min = 1, message = "password cannot be empty"))]
    pub password: String,

    #[validate(url(message = "url is not valid"))]
    pub url: String,
}
