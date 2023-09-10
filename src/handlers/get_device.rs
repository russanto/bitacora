use axum::{extract::{State, Path}, http::StatusCode, Json, response::{IntoResponse, Response}};

use crate::storage::storage::DeviceStorage;

use super::errors::ErrorResponse;

pub async fn handler<S: DeviceStorage>(
    Path(id): Path<String>,
    State(state): State<S>
) -> Response {
    match state.get_device(&id) {
        Ok(query_result) => {
            match query_result {
                Some(device) => (StatusCode::OK, Json(device)).into_response(),
                None => ErrorResponse::not_found("Device").into_response()
            }
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(())).into_response()
    }
}