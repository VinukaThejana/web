use crate::util::verify;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub date: String,
    pub description: String,
    pub tags: Vec<String>,
}
impl Post {
    pub fn new(
        id: i32,
        title: String,
        slug: String,
        date: String,
        description: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id,
            title,
            slug,
            date,
            description,
            tags,
        }
    }
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct PartialPost {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub summary: String,
    pub tags: String,
    pub date: i32,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct PostModel {
    pub id: i32,
    pub title: String,
    pub seo_title: String,
    pub slug: String,
    pub photo_url: String,
    pub tags: String,
    pub summary: String,
    pub content: String,
    pub date: i32,
}

pub trait ToPosts {
    fn to_posts(self) -> Vec<Post>;
}

impl ToPosts for Vec<PartialPost> {
    fn to_posts(self) -> Vec<Post> {
        self.into_iter()
            .map(|post| {
                Post::new(
                    post.id,
                    post.title.clone(),
                    post.slug.clone(),
                    DateTime::<Utc>::from_timestamp(post.date.into(), 0)
                        .unwrap()
                        .format("%Y-%m-%d")
                        .to_string(),
                    post.summary.clone(),
                    post.tags
                        .split(",")
                        .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect())
                        .collect(),
                )
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AddPost {
    #[validate(length(
        min = 5,
        max = 50,
        message = "title must be between 5 and 50 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 5,
        max = 255,
        message = "seo_title must be between 5 and 255 characters"
    ))]
    pub seo_title: String,

    #[validate(custom(function = "verify::slug"))]
    pub slug: String,

    #[validate(length(
        min = 50,
        max = 160,
        message = "summary must be between 50 and 160 characters"
    ))]
    pub summary: String,

    #[validate(length(max = 255, message = "photo_url must be less than 255 characters"))]
    pub photo_url: String,

    #[validate(length(max = 50, message = "tags must be less than 50 characters"))]
    pub tags: String,

    #[validate(length(
        max = 100_000,
        message = "content must be less than 100_000 characters"
    ))]
    pub content: String,

    pub password: String,

    #[validate(length(min = 1, message = "not valid"))]
    #[serde(rename = "cf-turnstile-response")]
    pub cf_turnstile_response: String,

    pub date: u64,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct EditPost {
    pub id: i32,
    pub date: u64,

    #[validate(length(
        min = 5,
        max = 50,
        message = "title must be between 5 and 50 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 5,
        max = 255,
        message = "seo_title must be between 5 and 255 characters"
    ))]
    pub seo_title: String,

    #[validate(custom(function = "verify::slug"))]
    pub slug: String,

    #[validate(length(
        min = 50,
        max = 160,
        message = "summary must be between 50 and 160 characters"
    ))]
    pub summary: String,

    #[validate(length(max = 255, message = "photo_url must be less than 255 characters"))]
    pub photo_url: String,

    #[validate(length(max = 50, message = "tags must be less than 50 characters"))]
    pub tags: String,

    #[validate(length(
        max = 100_000,
        message = "content must be less than 100_000 characters"
    ))]
    pub content: String,

    pub password: String,

    #[validate(length(min = 1, message = "not valid"))]
    #[serde(rename = "cf-turnstile-response")]
    pub cf_turnstile_response: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DelPost {
    #[validate(custom(function = "verify::slug"))]
    pub slug: String,

    pub password: String,

    #[validate(length(min = 1, message = "not valid"))]
    #[serde(rename = "cf-turnstile-response")]
    pub cf_turnstile_response: String,
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct PartialPostWithSlug {
    pub id: i32,
    pub date: i32,
    pub slug: String,
}
