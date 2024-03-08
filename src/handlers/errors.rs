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

#[derive(Serialize)]
pub struct ErrorResponseBody {
    pub code: u16,
    pub message: String,
    pub description: String,
}

impl ErrorResponse {
    pub fn already_exists() -> Self {
        ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            body: ErrorResponseBody {
                code: 1001,
                message: format!("Resource already exists"),
                description: format!("Resource already exists"),
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
            },
        }
    }

    pub fn web3_error() -> Self {
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorResponseBody {
                code: 1004,
                message: String::from("Error submitting a web3 transaction"),
                description: format!("[TODO]A better description should be implemented"), //TODO: improve description
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
            BitacoraError::CompletedWithError(_) => unimplemented!(),
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
