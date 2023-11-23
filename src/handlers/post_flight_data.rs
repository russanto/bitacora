use axum::{
    extract::State,
    Json, response::{IntoResponse, Response}
};

use serde::{ Deserialize, Serialize };

use crate::{ SharedBitacora, state::errors::BitacoraError, storage::storage::FullStorage, web3::traits::Timestamper};
use crate::state::entities::{ FlightData, LocalizationPoint, };

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

pub async fn handler<S: FullStorage, T: Timestamper>(
    State(state): State<SharedBitacora<S, T>>,
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
            BitacoraError::AlreadyExists => ErrorResponse::already_exists().into_response(),
            BitacoraError::NotFound => ErrorResponse::not_found("Device").into_response(),
            BitacoraError::StorageError => ErrorResponse::storage_error().into_response(),
            BitacoraError::Web3Error => ErrorResponse::web3_error().into_response()
        }
    }
}

