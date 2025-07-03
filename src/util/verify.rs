use std::borrow::Cow;
use validator::ValidationError;

pub fn slug(slug: &str) -> Result<(), ValidationError> {
    let checks = [
        (slug.len() < 2, "slug must be greater than 2 characters"),
        (slug.len() > 30, "slug must be less than 30 characters"),
        (
            !slug
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "slug must contain only alphanumeric characters, hyphens, or underscores",
        ),
    ];

    for (not_valid, message) in checks {
        if not_valid {
            return Err(ValidationError::new("slug").with_message(Cow::Borrowed(message)));
        }
    }

    Ok(())
}

pub fn key(key: &str) -> Result<(), ValidationError> {
    let checks = [
        (key.len() < 2, "key must be greater than 2 characters"),
        (key.len() > 30, "key must be less than 20 characters"),
        (
            !key.chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "key must contain only alphanumeric characters, hyphens, or underscores",
        ),
    ];

    for (not_valid, message) in checks {
        if not_valid {
            return Err(ValidationError::new("key").with_message(Cow::Borrowed(message)));
        }
    }

    Ok(())
}
