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

impl From<entity::short::Model> for Link {
    fn from(value: entity::short::Model) -> Self {
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

impl ToLinks for Vec<entity::short::Model> {
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
pub struct DelShort {
    #[validate(custom(function = "verify::slug"))]
    pub key: String,

    pub password: String,

    #[validate(length(min = 1, message = "not valid"))]
    #[serde(rename = "cf-turnstile-response")]
    pub cf_turnstile_response: String,
}
