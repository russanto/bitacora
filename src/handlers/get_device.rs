use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::SharedBitacora;
use crate::{
    storage::storage::{DeviceStorage, FullStorage},
    web3::traits::Timestamper,
};

use super::errors::ErrorResponse;

pub async fn handler<S: FullStorage, T: Timestamper>(
    Path(id): Path<String>,
    State(state): State<SharedBitacora<S, T>>,
) -> Response {
    match state.get_device(&id) {
        Ok(device) => (StatusCode::OK, Json(device)).into_response(),
        Err(err) => ErrorResponse::from(err).into_response(),
    }
}
