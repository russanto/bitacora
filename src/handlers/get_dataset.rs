use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use super::errors::ErrorResponse;
use crate::SharedBitacora;
use crate::{
    storage::storage::{FlightDataStorage, FullStorage},
    web3::traits::Timestamper,
};

pub async fn handler<S: FullStorage, T: Timestamper>(
    Path(id): Path<String>,
    State(state): State<SharedBitacora<S, T>>,
) -> Response {
    match state.get_dataset(&id) {
        Ok(dataset) => (StatusCode::OK, Json(dataset)).into_response(),
        Err(error) => ErrorResponse::from(error).into_response(),
    }
}
