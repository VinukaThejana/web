use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use sea_orm::{DbErr, RuntimeErr};
use std::fmt::Display;
use validator::ValidationErrors;

use crate::pages::{notfound::NotFound, servererror::InternalServerError};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    NotFound(#[source] anyhow::Error),
    BadRequest(#[source] anyhow::Error),
    UniqueViolation(#[source] anyhow::Error),
    Unauthorized(#[source] anyhow::Error),
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
            Self::Validation(e) => {
                let message =
                    e.field_errors()
                        .iter()
                        .fold(String::new(), |acc, (field, errors)| {
                            let binding = errors
                                .iter()
                                .map(|err| {
                                    err.message
                                        .as_ref()
                                        .map(|msg| msg.to_string())
                                        .unwrap_or_else(|| String::from("invalid value"))
                                })
                                .collect::<Vec<String>>();

                            let fe = binding.first();
                            if fe.is_none() {
                                return acc;
                            }
                            let fe = fe.unwrap();

                            if acc.is_empty() {
                                format!(r#""{}": "{}""#, field, fe)
                            } else {
                                format!(r#"{}, "{}": "{}""#, acc, field, fe)
                            }
                        });

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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(error) => {
                log::error!("not found error: {:?}", error);
                (
                    StatusCode::NOT_FOUND,
                    Html(NotFound::default().render().unwrap()),
                )
                    .into_response()
            }
            Self::BadRequest(error) => {
                log::error!("bad request error: {:?}", error);
                todo!()
            }
            Self::UniqueViolation(error) => {
                log::error!("unique violation error: {:?}", error);
                todo!()
            }
            Self::Unauthorized(error) => {
                log::error!("unauthorized error: {:?}", error);
                todo!()
            }
            Self::Validation(validation_errors) => {
                log::error!(
                    "validation error: {}",
                    AppError::Validation(validation_errors)
                );
                todo!()
            }
            Self::Other(error) => {
                log::error!("internal server error: {:?}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(InternalServerError::default().render().unwrap()),
                )
                    .into_response()
            }
        }
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
