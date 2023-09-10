use axum::{extract::{State, Path}, http::StatusCode, Json, response::{IntoResponse, Response}};

use crate::storage::storage::FlightDataStorage;

use super::errors::ErrorResponse;

pub async fn handler<S: FlightDataStorage>(
    Path(id): Path<String>,
    State(state): State<S>
) -> Response {
    match state.get_flight_data(&id) {
        Ok(query_result) => {
            match query_result {
                Some(fd) => (StatusCode::OK, Json(fd)).into_response(),
                None => ErrorResponse::not_found("FlightData").into_response()
            }
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(())).into_response()
    }
}