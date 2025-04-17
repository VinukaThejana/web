use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct Payload {
    #[validate(length(min = 1, message = "path cannot be empty"))]
    pub path: String,

    #[validate(length(min = 1, message = "password cannot be empty"))]
    pub password: String,

    #[validate(length(min = 1, message = "not valid"))]
    #[serde(rename = "cf-turnstile-response")]
    pub cf_turnstile_response: String,
}
