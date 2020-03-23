use actix_threadpool::BlockingError;
use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use r2d2::Error as R2D2Error;
use std::{convert::From, fmt::Debug};
use thiserror::Error;
use tokio::task::JoinError;
use uuid::Error as ParseError;

use crate::logged_user::TRIGGER_DB_UPDATE;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("BadRequest: {0}")]
    BadRequest(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("DBError")]
    DbError(#[from] DBError),
    #[error("blocking error {0}")]
    BlockingError(String),
    #[error("R2D2 Error {0}")]
    R2D2Error(#[from] R2D2Error),
    #[error("JoinError {0}")]
    JoinError(#[from] JoinError),
}

// impl ResponseError trait allows to convert our errors into http responses
// with appropriate data
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            Self::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            Self::Unauthorized => {
                TRIGGER_DB_UPDATE.set();
                HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(
                        include_str!("../static/login.html")
                            .replace("main.css", "../auth/main.css")
                            .replace("main.js", "../auth/main.js"),
                    )
            }
            _ => {
                HttpResponse::InternalServerError().json("Internal Server Error, Please try later")
            }
        }
    }
}

// we can return early in our handlers if UUID provided by the user is not valid
// and provide a custom message
impl From<ParseError> for ServiceError {
    fn from(_: ParseError) -> Self {
        Self::BadRequest("Invalid UUID".into())
    }
}

impl<T: Debug> From<BlockingError<T>> for ServiceError {
    fn from(item: BlockingError<T>) -> Self {
        Self::BlockingError(item.to_string())
    }
}
