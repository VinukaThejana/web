use crate::pages::{notfound, servererror};
use askama::Template;
use axum::{
    Json,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use sea_orm::{DbErr, RuntimeErr};
use serde_json::json;
use std::fmt::Display;
use validator::ValidationErrors;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    NotFound(#[source] anyhow::Error),
    BadRequest(#[source] anyhow::Error),
    UniqueViolation(#[source] anyhow::Error),
    Unauthorized(#[source] anyhow::Error),
    CaptchaFailed(#[source] anyhow::Error),
    Validation(#[from] ValidationErrors),
    Other(#[from] anyhow::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(e) => write!(f, "{:?}", e),
            Self::BadRequest(e) => write!(f, "{:?}", e),
            Self::UniqueViolation(e) => write!(f, "{:?}", e),
            Self::Unauthorized(e) => write!(f, "{:?}", e),
            Self::CaptchaFailed(e) => write!(f, "{:?}", e),
            Self::Validation(e) => {
                let message = e
                    .field_errors()
                    .values()
                    .flat_map(|e| e.iter())
                    .filter_map(|err| {
                        err.message
                            .as_ref()
                            .map(|msg| msg.to_string())
                            .or(Some(String::from("invalid value")))
                    })
                    .next()
                    .unwrap_or(String::from("invalid value"));

                write!(f, "{}", message)
            }
            Self::Other(e) => write!(f, "{:?}", e),
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

    pub fn from_not_found_error<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::NotFound(anyhow::Error::new(e))
    }

    pub fn from_database_error(error: DbErr) -> Self {
        match error {
            DbErr::RecordNotFound(err) => Self::NotFound(anyhow::anyhow!(err)),
            err => {
                if is_unique_violation(&err) {
                    Self::UniqueViolation(err.into())
                } else {
                    Self::Other(err.into())
                }
            }
        }
    }
}

pub struct HtmlError(pub AppError);
impl From<AppError> for HtmlError {
    fn from(value: AppError) -> Self {
        HtmlError(value)
    }
}

pub struct JsonError(pub AppError);
impl From<AppError> for JsonError {
    fn from(value: AppError) -> Self {
        JsonError(value)
    }
}
impl From<ValidationErrors> for JsonError {
    fn from(err: ValidationErrors) -> Self {
        JsonError(AppError::Validation(err))
    }
}

impl IntoResponse for HtmlError {
    fn into_response(self) -> Response {
        match self.0 {
            AppError::NotFound(error) => {
                log::error!("not found error: {:?}", error);
                (
                    StatusCode::NOT_FOUND,
                    Html(notfound::Tmpl::default().render().unwrap()),
                )
                    .into_response()
            }
            AppError::BadRequest(error) => {
                log::error!("bad request error: {:?}", error);
                unimplemented!()
            }
            AppError::UniqueViolation(error) => {
                log::error!("unique violation error: {:?}", error);
                unimplemented!()
            }
            AppError::Unauthorized(error) => {
                log::error!("unauthorized error: {:?}", error);
                unimplemented!()
            }
            AppError::CaptchaFailed(error) => {
                log::error!("captcha failed error: {:?}", error);
                unimplemented!()
            }
            AppError::Validation(validation_errors) => {
                log::error!(
                    "validation error: {}",
                    AppError::Validation(validation_errors)
                );
                unimplemented!()
            }
            AppError::Other(error) => {
                log::error!("internal server error: {:?}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(servererror::Tmpl::default().render().unwrap()),
                )
                    .into_response()
            }
        }
    }
}

impl IntoResponse for JsonError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            AppError::NotFound(error) => {
                log::error!("not found error: {:?}", error);
                (StatusCode::NOT_FOUND, String::from("not found"))
            }
            AppError::BadRequest(error) => {
                log::error!("bad request error: {:?}", error);
                (StatusCode::BAD_REQUEST, String::from("bad request"))
            }
            AppError::UniqueViolation(error) => {
                log::error!("unique violation error: {:?}", error);
                (StatusCode::BAD_REQUEST, String::from("unique violation"))
            }
            AppError::Unauthorized(error) => {
                log::error!("unauthorized error: {:?}", error);
                (StatusCode::UNAUTHORIZED, String::from("unauthorized"))
            }
            AppError::CaptchaFailed(error) => {
                log::error!("captcha failed error: {:?}", error);
                (StatusCode::BAD_REQUEST, String::from("captcha failed"))
            }
            AppError::Validation(ve) => {
                let ve = AppError::Validation(ve);
                log::error!("validation errors : {}", ve);
                (StatusCode::BAD_REQUEST, format!("{ve}"))
            }
            AppError::Other(error) => {
                log::error!("internal server error: {:?}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("internal server error"),
                )
            }
        };

        (status, Json(json!({ "status": message }))).into_response()
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
