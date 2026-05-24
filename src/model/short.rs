use crate::util::verify;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug)]
pub struct Link {
    pub long_url: String,
    pub key: String,
    pub description: String,
    pub views: u32,
}

#[derive(sqlx::FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct ShortModel {
    pub id: i32,
    pub long_url: String,
    pub key: String,
    pub description: String,
    pub views: i32,
    pub created_at: chrono::NaiveDateTime,
}

impl From<ShortModel> for Link {
    fn from(value: ShortModel) -> Self {
        Self {
            long_url: value.long_url,
            key: value.key,
            description: value.description,
            views: value.views.try_into().unwrap(),
        }
    }
}

pub trait ToLinks {
    fn to_links(self) -> Vec<Link>;
}

impl ToLinks for Vec<ShortModel> {
    fn to_links(self) -> Vec<Link> {
        self.into_iter().map(Link::from).collect()
    }
}

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

#[derive(Default, Debug, Serialize, Deserialize, Validate)]
pub struct AddShortAPI {
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
}

#[derive(Default, Debug, Serialize, Deserialize, Validate)]
pub struct DelShort {
    #[validate(custom(function = "verify::slug"))]
    pub key: String,

    pub password: String,

    #[validate(length(min = 1, message = "not valid"))]
    #[serde(rename = "cf-turnstile-response")]
    pub cf_turnstile_response: String,
}
