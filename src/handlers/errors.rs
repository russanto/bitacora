use axum::{http::StatusCode, Json, response::IntoResponse};
use serde::Serialize;


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
    pub fn already_exists() -> Self {
        ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            body: ErrorResponseBody {
                code: 1001,
                message: String::from("Resource already exists"),
                description: String::from("The resource you are trying to create already exists, or \
                                            clashes with the Id of an existing resource")
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