use std::borrow::Cow;
use validator::ValidationError;

pub fn slug(slug: &str) -> Result<(), ValidationError> {
    let checks = [
        (slug.len() < 5, "slug must be greater than 5 characters"),
        (slug.len() > 30, "slug must be less than 30 characters"),
        (
            !slug.chars().all(|c| c.is_alphanumeric() || c == '-'),
            "slug must contain only alphanumeric characters or hyphens",
        ),
    ];

    for (not_valid, message) in checks {
        if not_valid {
            return Err(ValidationError::new("slug").with_message(Cow::Borrowed(message)));
        }
    }

    Ok(())
}
