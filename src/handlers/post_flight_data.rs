use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use base64::{engine::general_purpose::STANDARD, DecodeError, Engine as _};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::state::entities::{FlightData, LocalizationPoint};
use crate::{
    state::entities::FlightDataId, storage::storage::FullStorage, web3::traits::Timestamper,
    SharedBitacora,
};

use super::errors::ErrorResponse;

#[derive(Debug, Deserialize)]
pub struct POSTFlightDataRequest {
    device_id: String,
    timestamp: u64,
    localization: LocalizationPoint,
    payload: String,
    signature: String,
}

pub enum InputFlightDataError {
    BadPayloadData(DecodeError),
}

impl TryFrom<POSTFlightDataRequest> for FlightData {
    type Error = InputFlightDataError;

    fn try_from(value: POSTFlightDataRequest) -> Result<Self, Self::Error> {
        let payload = match STANDARD.decode(value.payload) {
            Ok(payload) => payload,
            Err(err) => return Err(InputFlightDataError::BadPayloadData(err)),
        };
        Ok(FlightData {
            id: FlightDataId::new(value.timestamp, &value.device_id, &value.localization),
            signature: value.signature,
            timestamp: value.timestamp,
            localization: value.localization,
            payload,
        })
        // TODO: add parameters validation
        // TODO: add FlightData validation
    }
}

#[derive(Serialize)]
pub struct POSTFlightDataResponse {
    pub id: String,
    pub dataset_id: String,
}

pub async fn handler<S: FullStorage, T: Timestamper>(
    State(state): State<SharedBitacora<S, T>>,
    Json(payload): Json<POSTFlightDataRequest>,
) -> Response {
    tracing::debug!("received flight data {:?}", payload);
    let device_id = payload.device_id.clone(); //clone device id so that can move values of payload to not copy them
    let flight_data = match FlightData::try_from(payload) {
        Ok(fd) => fd,
        Err(err) => match err {
            InputFlightDataError::BadPayloadData(err) => {
                warn!(
                    device_id = device_id,
                    "Failed to decode input payload for new FlightData"
                );
                return ErrorResponse::bad_input("payload", Some(&err.to_string())).into_response();
            }
        },
    };
    match state.new_flight_data(&flight_data, &device_id).await {
        Ok(dataset) => Json(POSTFlightDataResponse {
            id: flight_data.id.into(),
            dataset_id: dataset.id,
        })
        .into_response(),
        Err(new_fd_error) => {
            error!(
                device_id = device_id,
                flight_data_id = flight_data.id.to_string(),
                "{}",
                new_fd_error
            );
            ErrorResponse::from(new_fd_error).into_response()
        }
    }
}
