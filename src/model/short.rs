use crate::util::verify;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Default, Debug, Serialize, Deserialize, Validate)]
pub struct ShortKey {
    #[validate(custom(function = "verify::slug"))]
    pub key: String,
}
