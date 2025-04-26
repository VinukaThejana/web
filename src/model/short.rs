use crate::util::verify;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Default, Debug, Serialize, Deserialize, Validate)]
pub struct ShortKey {
    #[validate(custom(function = "verify::slug"))]
    pub key: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Validate)]
pub struct AddShort {
    #[validate(url)]
    pub long_url: String,

    #[validate(custom(function = "verify::slug"))]
    pub key: String,

    #[validate(length(
        min = 1,
        max = 160,
        message = "description must be between 1 and 160 characters"
    ))]
    pub description: String,

    pub password: String,

    #[validate(length(min = 1, message = "not valid"))]
    #[serde(rename = "cf-turnstile-response")]
    pub cf_turnstile_response: String,
}
