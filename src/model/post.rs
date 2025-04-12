use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

pub trait ToPosts {
    fn to_posts(self) -> Vec<Post>;
}

impl ToPosts for Vec<entity::post::Model> {
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
