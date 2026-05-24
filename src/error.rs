use crate::pages::status::{notfound, servererror};
use askama::Template;
use axum::{
    Json,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use sea_orm::{DbErr, RuntimeErr};
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
        Self::NotFound {
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
            _ => None,
        }
    }

    fn internal_message(&self) -> Option<String> {
        match self {
            AppError::Other(e) => Some(format!("{:#}", e)),
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
            _ => None,
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
            .unwrap_or_else(|| String::from("something went wrong"));
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

    pub fn from_database_error(error: DbErr) -> Self {
        match error {
            DbErr::RecordNotFound(err) => Self::NotFound {
                user_message: "record not found".to_string(),
                internal_message: None,
                source: Some(anyhow::anyhow!(err)),
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
    fn from(err: E) -> Self {
        HtmlError(err.into())
    }
}

pub struct JsonError(pub AppError);

impl<E> From<E> for JsonError
where
    E: Into<AppError>,
{
    fn from(err: E) -> Self {
        JsonError(err.into())
    }
}

impl IntoResponse for HtmlError {
    fn into_response(self) -> Response {
        let html = match self.0 {
            AppError::NotFound { .. } => notfound::Tmpl::default().render().unwrap(),
            AppError::BadRequest { .. } => unimplemented!(),
            AppError::UniqueViolation { .. } => unimplemented!(),
            AppError::Unauthorized { .. } => unimplemented!(),
            AppError::CaptchaFailed { .. } => unimplemented!(),
            AppError::Validation(_) => unimplemented!(),
            AppError::Other(_) => servererror::Tmpl::default().render().unwrap(),
        };

        self.0.log();

        (self.0.get_status_code(), Html(html)).into_response()
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

fn is_unique_violation(err: &DbErr) -> bool {
    match err {
        DbErr::Query(RuntimeErr::SqlxError(error)) => {
            if let Some(db_error) = error.as_database_error() {
                if let Some(code) = db_error.code() {
                    code == "23505"
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
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
