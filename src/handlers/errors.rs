use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::state::errors::BitacoraError;
use crate::storage::errors::Error as StorageError;

#[derive(Debug)]
pub enum Error {}

pub struct ErrorResponse {
    pub status: StatusCode,
    pub body: ErrorResponseBody,
}

type ErrorCode = u16;

#[derive(Serialize)]
pub struct ErrorResponseBody {
    pub code: ErrorCode,
    pub message: String,
    pub description: String,
    pub nested: Option<Box<ErrorResponseBody>>,
}

impl ErrorResponse {
    pub fn already_exists() -> Self {
        ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            body: ErrorResponseBody {
                code: 1001,
                message: format!("Resource already exists"),
                description: format!("Resource already exists"),
                nested: None,
            },
        }
    }

    pub fn not_found(what: &str) -> Self {
        ErrorResponse {
            status: StatusCode::NOT_FOUND,
            body: ErrorResponseBody {
                code: 1002,
                message: String::from("Resource not found"),
                description: format!("The following resorce was not found: {}", what),
                nested: None,
            },
        }
    }

    pub fn bad_input(parameter: &str, reason: Option<&str>) -> Self {
        ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            body: ErrorResponseBody {
                code: 1003,
                message: format!("Error on input parameter: {}", parameter),
                description: format!(
                    "The parameter produced the following error: {}",
                    reason.unwrap_or_default()
                ),
                nested: None,
            },
        }
    }

    pub fn storage_error() -> Self {
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorResponseBody {
                code: 1003,
                message: String::from("Error on internal data storage"),
                description: format!("[TODO]A better description should be implemented"), //TODO: improve description
                nested: None,
            },
        }
    }

    pub fn web3_error() -> Self {
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorResponseBody {
                code: 1004,
                message: String::from("Error with the web3 interaction"),
                description: format!(
                    "The Web3 operation failed. Nested error should clarify the context"
                ), //TODO: improve description
                nested: None,
            },
        }
    }

    pub fn completed_with_error(err: BitacoraError) -> Self {
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorResponseBody {
                code: 1100,
                message: String::from("Operation completed with errors"),
                description: format!(
                    "The operation completed with the following error: {:?}",
                    err
                ),
                nested: Option::Some(Box::new(ErrorResponse::from(err).body)),
            },
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl From<BitacoraError> for ErrorResponse {
    fn from(value: BitacoraError) -> Self {
        match value {
            BitacoraError::Web3Error => ErrorResponse::web3_error(),
            BitacoraError::StorageError(err) => err.into(),
            BitacoraError::BadId(_id_err) => ErrorResponse::bad_input("id", None),
            BitacoraError::CompletedWithError(err) => ErrorResponse::completed_with_error(*err),
        }
    }
}

impl From<StorageError> for ErrorResponse {
    fn from(value: StorageError) -> Self {
        match value {
            StorageError::NotFound(entity) => ErrorResponse::not_found(entity.to_string().as_str()),
            StorageError::AlreadyExists => ErrorResponse::already_exists(),
            _ => ErrorResponse::storage_error(),
        }
    }
}
