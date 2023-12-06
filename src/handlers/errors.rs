use axum::{http::StatusCode, Json, response::IntoResponse};
use serde::Serialize;

use crate::state::{errors::BitacoraError, entities::Entity};


#[derive(Debug)]
pub enum Error {

}

pub struct ErrorResponse {
    pub status: StatusCode,
    pub body: ErrorResponseBody
}

#[derive(Serialize)]
pub struct ErrorResponseBody {
    pub code: u16,
    pub message: String,
    pub description: String
}

impl ErrorResponse {
    pub fn already_exists(entity: Entity, id: String) -> Self {
        ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            body: ErrorResponseBody {
                code: 1001,
                message: format!("{} already exists", entity),
                description: format!("Resource {} already exists", id)
            }
        }
    }

    pub fn not_found(what: &str) -> Self {
        ErrorResponse {
            status: StatusCode::NOT_FOUND,
            body: ErrorResponseBody {
                code: 1002,
                message: String::from("Resource not found"),
                description: format!("The following resorce was not found: {}", what)
            } 
        }
    }

    pub fn bad_input(parameter: &str, reason: Option<&str>) -> Self {
        ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            body: ErrorResponseBody {
                code: 1003,
                message: format!("Error on input parameter: {}", parameter),
                description: format!("The parameter produced the following error: {}", reason.unwrap_or_default())
            }
        }
    }

    pub fn storage_error() -> Self {
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorResponseBody {
                code: 1003,
                message: String::from("Error on internal data storage"),
                description: format!("[TODO]A better description should be implemented") //TODO: improve description
            } 
        }
    }

    pub fn web3_error() -> Self {
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorResponseBody {
                code: 1004,
                message: String::from("Error submitting a web3 transaction"),
                description: format!("[TODO]A better description should be implemented") //TODO: improve description
            } 
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
            BitacoraError::AlreadyExists(entity, id) => ErrorResponse::already_exists(entity, id),
            BitacoraError::Web3Error => ErrorResponse::web3_error(),
            BitacoraError::NotFound => ErrorResponse::not_found(&String::from("CHANGE ME")),
            BitacoraError::StorageError(_) => ErrorResponse::storage_error(),
            BitacoraError::BadIdFormat => ErrorResponse::bad_input("id", None)
        }
    }
}