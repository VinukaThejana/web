use aws_sdk_s3::types::Object;
use aws_smithy_types::date_time::Format::DateTime;

pub struct AwsObject {
    pub path: String,
    pub modified: String,
    pub size: usize,
}

impl From<&Object> for AwsObject {
    fn from(value: &Object) -> Self {
        Self {
            path: value.key().unwrap_or_default().to_string(),
            modified: value
                .last_modified()
                .map_or_else(|| "Unknown".to_string(), |date| date.fmt(DateTime).unwrap()),
            size: value.size().unwrap_or_default() as usize,
        }
    }
}
