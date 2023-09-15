use axum::{extract::{State, Path}, http::StatusCode, Json, response::{IntoResponse, Response}};

use crate::storage::storage::DatasetStorage;

use super::errors::ErrorResponse;

pub async fn handler<S: DatasetStorage>(
    Path(id): Path<String>,
    State(state): State<S>
) -> Response {
    match state.get_dataset(&id) {
        Ok(query_result) => {
            match query_result {
                Some(dataset) => (StatusCode::OK, Json(dataset)).into_response(),
                None => ErrorResponse::not_found("Dataset").into_response()
            }
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(())).into_response()
    }
}