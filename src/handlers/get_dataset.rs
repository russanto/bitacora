use axum::{extract::{State, Path}, http::StatusCode, Json, response::{IntoResponse, Response}};

use crate::{storage::storage::{FlightDataStorage, FullStorage}, web3::traits::Timestamper};
use crate::SharedBitacora;
use super::errors::ErrorResponse;

pub async fn handler<S: FullStorage, T: Timestamper>(
    Path(id): Path<String>,
    State(state): State<SharedBitacora<S, T>>
) -> Response {
    match state.get_dataset(&id) {
        Ok(dataset) =>  (StatusCode::OK, Json(dataset)).into_response(),
        Err(error) => ErrorResponse::from(error).into_response()
    }
}