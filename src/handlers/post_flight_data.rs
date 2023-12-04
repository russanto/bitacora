use axum::{
    extract::State,
    Json, response::{IntoResponse, Response}
};

use base64::{DecodeError, Engine as _, engine::general_purpose::STANDARD_NO_PAD};
use serde::{ Deserialize, Serialize };

use crate::{ SharedBitacora, state::{errors::BitacoraError, entities::FlightDataId}, storage::storage::FullStorage, web3::traits::Timestamper};
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

pub enum InputFlightDataError {
    BadPayloadData(DecodeError)
}

impl TryFrom<POSTFlightDataRequest> for FlightData {

    type Error = InputFlightDataError;
    
    fn try_from(value: POSTFlightDataRequest) -> Result<Self, Self::Error> {
        let payload = match STANDARD_NO_PAD.decode(value.payload) {
            Ok(payload) => payload,
            Err(err) => return Err(InputFlightDataError::BadPayloadData(err))
        };
        Ok(FlightData {
            id: FlightDataId::new(value.timestamp, &value.device_id),
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
    pub dataset_id: String
}

pub async fn handler<S: FullStorage, T: Timestamper>(
    State(state): State<SharedBitacora<S, T>>,
    Json(payload): Json<POSTFlightDataRequest>
) -> Response {
    tracing::debug!("received flight data {:?}", payload);
    let device_id = payload.device_id.clone(); //clone device id so that can move values of payload to not copy them
    let flight_data = match FlightData::try_from(payload) {
        Ok(fd) => fd,
        Err(err) => match err {
            InputFlightDataError::BadPayloadData(_) => return ErrorResponse::bad_input("payload", None).into_response()
        }
    };
    match state.new_flight_data(&flight_data, &device_id).await {
        Ok(dataset) => Json(POSTFlightDataResponse {
            id: flight_data.id.into(),
            dataset_id: dataset.id
        }).into_response(),
        Err(new_fd_error) => match new_fd_error {
            BitacoraError::AlreadyExists(entity, id) => ErrorResponse::already_exists(entity, id).into_response(),
            BitacoraError::NotFound => ErrorResponse::not_found("Device").into_response(),
            BitacoraError::StorageError(_) => ErrorResponse::storage_error().into_response(),
            BitacoraError::Web3Error => ErrorResponse::web3_error().into_response(),
            BitacoraError::BadIdFormat => ErrorResponse::bad_input("device_id", Some("Bad Device Id")).into_response() //this should be unreachable
        }
    }
}

