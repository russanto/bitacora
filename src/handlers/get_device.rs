use axum::{extract::{State, Path}, http::StatusCode, Json, response::{IntoResponse, Response}};

use crate::{storage::storage::{DeviceStorage, FullStorage}, web3::traits::Timestamper};
use crate::SharedBitacora;

use super::errors::ErrorResponse;

pub async fn handler<S: FullStorage, T: Timestamper>(
    Path(id): Path<String>,
    State(state): State<SharedBitacora<S, T>>
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