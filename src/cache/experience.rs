use chrono::Duration;

use crate::config::ENV;

pub fn gck_experiences() -> String {
    format!("{}:experiences", &ENV.redis_schema)
}
pub fn gct_experiences() -> i64 {
    Duration::days(30).num_seconds()
}
