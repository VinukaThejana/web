use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct ContactUs {
    #[validate(email(message = "provide a valid email"))]
    pub email: String,

    #[validate(length(
        min = 5,
        max = 100,
        message = "name must be between 5 and 100 characters"
    ))]
    pub name: String,

    #[validate(length(
        min = 10,
        max = 1000,
        message = "message must be between 10 and 1000 characters"
    ))]
    pub message: String,
}
