use chrono::Duration;

use crate::config::ENV;

pub fn gck_for_slug(slug: &str) -> String {
    format!("{}:post:{}", &ENV.redis_schema, slug)
}

pub fn gct_for_slug() -> i64 {
    Duration::days(30).num_seconds()
}

pub fn gck_for_home() -> String {
    format!("{}:latest_posts", &ENV.redis_schema)
}
pub fn gct_for_home() -> i64 {
    Duration::days(30).num_seconds()
}

pub fn gck_for_total() -> String {
    format!("{}:blog:sum", &ENV.redis_schema)
}
pub fn gct_for_total() -> i64 {
    Duration::days(30).num_seconds()
}

pub fn gck_for_page(page: u64) -> String {
    format!("{}:blog:{}", &ENV.redis_schema, page)
}
pub fn gct_for_page() -> i64 {
    Duration::days(30).num_seconds()
}
