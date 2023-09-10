use axum::{
    http::StatusCode,
    extract::State,
    Json, response::{IntoResponse, Response}
};

use serde::{ Deserialize, Serialize };

use crate::{storage::storage::{DeviceStorage, FlightDataStorage}, SharedBitacora, state::errors::NewFlightDataError};
use crate::state::entities::{ Device, FlightData, LocalizationPoint, PublicKey };

use super::errors::ErrorResponse;

#[derive(Debug, Deserialize)]
pub struct POSTFlightDataRequest {
    device_id: String,
    timestamp: u64,
    localization: LocalizationPoint,
    payload: String,
    signature: String
}

impl From<POSTFlightDataRequest> for FlightData {
    fn from(value: POSTFlightDataRequest) -> Self {
        FlightData {
            id: FlightData::id(value.timestamp, &value.device_id),
            signature: value.signature,
            timestamp: value.timestamp,
            localization: value.localization,
            payload: value.payload,
        }
        // TODO: add parameters validation
        // TODO: add FlightData validation
    }
}

#[derive(Serialize)]
pub struct POSTFlightDataResponse {
    pub id: String,
    pub dataset_id: String
}

pub async fn handler(
    State(state): State<SharedBitacora>,
    Json(payload): Json<POSTFlightDataRequest>
) -> Response {
    tracing::debug!("received flight data {:?}", payload);
    let device_id = payload.device_id.clone(); //clone device id so that can move values of payload to not copy them
    let flight_data = FlightData::from(payload);
    match state.new_flight_data(&flight_data, &device_id) {
        Ok(dataset) => Json(POSTFlightDataResponse {
            id: flight_data.id,
            dataset_id: dataset.id
        }).into_response(),
        Err(new_fd_error) => match new_fd_error {
            NewFlightDataError::AlreadyExists => ErrorResponse::already_exists().into_response(),
            NewFlightDataError::DeviceNotFound => ErrorResponse::not_found("Device").into_response(),
            NewFlightDataError::StorageError => ErrorResponse::storage_error().into_response()
        }
    }
}

