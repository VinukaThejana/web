use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use std::fmt::Display;
use validator::ValidationErrors;

use crate::pages::notfound::NotFound;

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
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(error) => (
                StatusCode::NOT_FOUND,
                Html(NotFound::default().render().unwrap()),
            )
                .into_response(),
            Self::BadRequest(error) => todo!(),
            Self::UniqueViolation(error) => todo!(),
            Self::Unauthorized(error) => todo!(),
            Self::Validation(validation_errors) => todo!(),
            Self::Other(error) => todo!(),
        }
    }
}
