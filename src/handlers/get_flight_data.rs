use axum::{extract::{State, Path}, http::StatusCode, Json, response::{IntoResponse, Response}};
use serde::Serialize;

use crate::state::entities::{FlightData, DatasetId};
use crate::{storage::storage::{FlightDataStorage, FullStorage}, web3::traits::{Timestamper, Web3Info}, state::entities::{FlightDataId, LocalizationPoint}};
use crate::SharedBitacora;

use super::errors::ErrorResponse;

#[derive(Serialize)]
pub struct FlightDataResponse {
    flight_data: FlightData,
    dataset_id: DatasetId,
    web3: Option<Web3Info>
}

pub async fn handler<S: FullStorage, T: Timestamper>(
    Path(id): Path<String>,
    State(state): State<SharedBitacora<S, T>>
) -> Response {
    let fd_id = match FlightDataId::try_from(id) {
        Ok(fd_id) => fd_id,
        Err(_) => return ErrorResponse::bad_input("id", Some("Can't decode Id")).into_response()
    };
    let fd = match state.get_flight_data(&fd_id) {
        Ok(fd) => fd,
        Err(err) => return ErrorResponse::from(err).into_response()
    };
    let dataset = match state.get_flight_data_dataset(&fd_id) {
        Ok(dataset) => dataset,
        Err(_) => return ErrorResponse::storage_error().into_response()
    };
    let fd_receipt = match state.get_flight_data_receipt(&fd) {
        Ok(receipt) => receipt,
        Err(_) => return ErrorResponse::web3_error().into_response()
    };
    let response = FlightDataResponse {
        flight_data: fd,
        dataset_id: dataset.id,
        web3: Some(fd_receipt)
    };
    (StatusCode::OK, Json(response)).into_response()
}