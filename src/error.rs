use crate::pages::status::{notfound, servererror};
use askama::Template;
use axum::{
    Json,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use validator::ValidationErrors;

pub trait StdErrorExt: std::error::Error + Send + Sync + 'static {}
impl<T> StdErrorExt for T where T: std::error::Error + Send + Sync + 'static {}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{user_message}")]
    NotFound {
        user_message: String,
        internal_message: Option<String>,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{user_message}")]
    BadRequest {
        user_message: String,
        internal_message: Option<String>,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{user_message}")]
    UniqueViolation {
        user_message: String,
        internal_message: Option<String>,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{user_message}")]
    Unauthorized {
        user_message: String,
        internal_message: Option<String>,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{user_message}")]
    CaptchaFailed {
        user_message: String,
        internal_message: Option<String>,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{0}")]
    Validation(#[from] ValidationErrors),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest {
            user_message: msg.into(),
            internal_message: None,
            source: None,
        }
    }

    pub fn bad_request_with_source<E>(
        msg: impl Into<String>,
        internal_message: Option<String>,
        source: E,
    ) -> Self
    where
        E: StdErrorExt,
    {
        Self::BadRequest {
            user_message: msg.into(),
            internal_message,
            source: Some(anyhow::Error::new(source)),
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound {
            user_message: msg.into(),
            internal_message: None,
            source: None,
        }
    }

    pub fn not_found_with_source<E>(
        msg: impl Into<String>,
        internal_message: Option<String>,
        source: E,
    ) -> Self
    where
        E: StdErrorExt,
    {
        Self::NotFound {
            user_message: msg.into(),
            internal_message,
            source: Some(anyhow::Error::new(source)),
        }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized {
            user_message: msg.into(),
            internal_message: None,
            source: None,
        }
    }

    pub fn unauthorized_with_source<E>(
        msg: impl Into<String>,
        internal_message: Option<String>,
        source: E,
    ) -> Self
    where
        E: StdErrorExt,
    {
        Self::Unauthorized {
            user_message: msg.into(),
            internal_message,
            source: Some(anyhow::Error::new(source)),
        }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::UniqueViolation {
            user_message: msg.into(),
            internal_message: None,
            source: None,
        }
    }

    pub fn conflict_with_source<E>(
        msg: impl Into<String>,
        internal_message: Option<String>,
        source: E,
    ) -> Self
    where
        E: StdErrorExt,
    {
        Self::UniqueViolation {
            user_message: msg.into(),
            internal_message,
            source: Some(anyhow::Error::new(source)),
        }
    }

    pub fn captcha(msg: impl Into<String>) -> Self {
        Self::CaptchaFailed {
            user_message: msg.into(),
            internal_message: None,
            source: None,
        }
    }

    pub fn captcha_with_source<E>(
        msg: impl Into<String>,
        internal_message: Option<String>,
        source: E,
    ) -> Self
    where
        E: StdErrorExt,
    {
        Self::CaptchaFailed {
            user_message: msg.into(),
            internal_message,
            source: Some(anyhow::Error::new(source)),
        }
    }
}

impl AppError {
    pub fn from_generic_error<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Other(anyhow::Error::new(e))
    }

    fn source_error(&self) -> Option<&anyhow::Error> {
        match self {
            AppError::BadRequest { source, .. }
            | AppError::NotFound { source, .. }
            | AppError::UniqueViolation { source, .. }
            | AppError::Unauthorized { source, .. }
            | AppError::CaptchaFailed { source, .. } => source.as_ref(),
            AppError::Other(e) => Some(e),
            _ => None,
        }
    }

    fn internal_message(&self) -> Option<String> {
        match self {
            AppError::Other(e) => Some(format!("{:#}", e)),
            AppError::Validation(errs) => Some(format!("{:?}", errs)),
            AppError::BadRequest {
                internal_message, ..
            }
            | AppError::NotFound {
                internal_message, ..
            }
            | AppError::UniqueViolation {
                internal_message, ..
            }
            | AppError::Unauthorized {
                internal_message, ..
            }
            | AppError::CaptchaFailed {
                internal_message, ..
            } => internal_message.clone(),
        }
    }

    fn get_tag(&self) -> String {
        match self {
            AppError::BadRequest { .. } => String::from("bad_request"),
            AppError::NotFound { .. } => String::from("not_found"),
            AppError::UniqueViolation { .. } => String::from("conflict"),
            AppError::Unauthorized { .. } => String::from("unauthorized"),
            AppError::CaptchaFailed { .. } => String::from("captcha_failed"),
            AppError::Validation(_) => String::from("validation"),
            AppError::Other(_) => String::from("other"),
        }
    }

    fn get_user_message(&self) -> String {
        match self {
            AppError::BadRequest { user_message, .. }
            | AppError::NotFound { user_message, .. }
            | AppError::UniqueViolation { user_message, .. }
            | AppError::Unauthorized { user_message, .. }
            | AppError::CaptchaFailed { user_message, .. } => user_message.to_string(),
            AppError::Validation(errs) => errs
                .field_errors()
                .values()
                .flat_map(|v| v.iter())
                .flat_map(|e| e.message.as_ref().map(|m| m.to_string()))
                .next()
                .unwrap_or_else(|| "invalid value".to_string()),
            AppError::Other(_) => String::from("something went wrong"),
        }
    }

    fn get_status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::UniqueViolation { .. } => StatusCode::BAD_REQUEST,
            AppError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            AppError::CaptchaFailed { .. } => StatusCode::BAD_REQUEST,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn log(&self) {
        let tag = self.get_tag();
        let msg = self
            .internal_message()
            .unwrap_or_else(|| self.get_user_message());
        let source = self.source_error();

        match self {
            AppError::Other(_) => {
                if let Some(source) = source {
                    log::error!("AppError [{}]: {} | source: {:#?}", tag, msg, source);
                } else {
                    log::error!("AppError [{}]: {}", tag, msg);
                }
            }
            _ => {
                if let Some(source) = source {
                    log::warn!("AppError [{}]: {} | source: {:#?}", tag, msg, source);
                } else {
                    log::warn!("AppError [{}]: {}", tag, msg);
                }
            }
        }
    }

    pub fn from_database_error(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::NotFound {
                user_message: "record not found".to_string(),
                internal_message: None,
                source: Some(anyhow::anyhow!(error)),
            },
            err => {
                if is_unique_violation(&err) {
                    Self::UniqueViolation {
                        user_message: "unique violation".to_string(),
                        internal_message: None,
                        source: Some(anyhow::anyhow!(err)),
                    }
                } else {
                    Self::Other(err.into())
                }
            }
        }
    }
}

pub struct HtmlError(pub AppError);

impl<E> From<E> for HtmlError
where
    E: Into<AppError>,
{
    fn from(e: E) -> Self {
        HtmlError(e.into())
    }
}

impl IntoResponse for HtmlError {
    fn into_response(self) -> Response {
        let html = match self.0 {
            AppError::NotFound { .. } => notfound::Tmpl::default().render().unwrap(),
            _ => servererror::Tmpl::default().render().unwrap(),
        };

        self.0.log();

        (self.0.get_status_code(), Html(html)).into_response()
    }
}

pub struct JsonError(pub AppError);

impl<E> From<E> for JsonError
where
    E: Into<AppError>,
{
    fn from(e: E) -> Self {
        JsonError(e.into())
    }
}

impl IntoResponse for JsonError {
    fn into_response(self) -> Response {
        self.0.log();

        (
            self.0.get_status_code(),
            [(header::CONTENT_TYPE, "application/json")],
            Json(json!({
                "status": self.0.get_tag(),
                "message": self.0.get_user_message(),
            })),
        )
            .into_response()
    }
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            return code == "23505"; // PostgreSQL unique violation error code
        }
    }
    false
}

macro_rules! impl_from_error {
    ($($t:ty),+ $(,)?) => {
        $(
            impl From<$t> for AppError {
                fn from(err: $t) -> Self {
                    Self::Other(anyhow::Error::new(err))
                }
            }
        )+
    };
}

impl_from_error!(
    std::io::Error,
    base64::DecodeError,
    std::string::FromUtf8Error,
    reqwest::Error,
    reqwest::header::InvalidHeaderValue,
    reqwest::header::ToStrError,
    redis::RedisError,
);

pub trait AppErrorOptionExt<T> {
    fn not_found_msg(self, msg: &str) -> Result<T, AppError>;
    fn bad_request_msg(self, msg: &str) -> Result<T, AppError>;
    fn unauthorized_msg(self, msg: &str) -> Result<T, AppError>;
    fn captcha_msg(self, msg: &str) -> Result<T, AppError>;
}

impl<T> AppErrorOptionExt<T> for Option<T> {
    fn not_found_msg(self, msg: &str) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::not_found(msg))
    }
    fn bad_request_msg(self, msg: &str) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::bad_request(msg))
    }
    fn unauthorized_msg(self, msg: &str) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::unauthorized(msg))
    }
    fn captcha_msg(self, msg: &str) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::captcha(msg))
    }
}

pub trait AppErrorResultExt<T> {
    fn into_bad_request(self) -> Result<T, AppError>;
    fn into_not_found(self) -> Result<T, AppError>;
    fn into_conflict(self) -> Result<T, AppError>;
    fn into_unauthorized(self) -> Result<T, AppError>;
    fn into_captcha(self) -> Result<T, AppError>;
}

impl<T, E> AppErrorResultExt<T> for Result<T, E>
where
    E: StdErrorExt,
{
    fn into_bad_request(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::bad_request_with_source(e.to_string(), None, e))
    }
    fn into_not_found(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::not_found_with_source(e.to_string(), None, e))
    }
    fn into_conflict(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::conflict_with_source(e.to_string(), None, e))
    }
    fn into_unauthorized(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::unauthorized_with_source(e.to_string(), None, e))
    }
    fn into_captcha(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::captcha_with_source(e.to_string(), None, e))
    }
}
