use chrono::Duration;

use crate::config::ENV;

pub fn gck_projects() -> String {
    format!("{}:projects", &ENV.redis_schema)
}
pub fn gct_projects() -> i64 {
    Duration::days(30).num_seconds()
}
