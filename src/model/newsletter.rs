use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct SignUp {
    #[validate(email(message = "not valid"))]
    pub email: String,
}
