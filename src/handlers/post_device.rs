use axum::{
    http::StatusCode,
    extract::State,
    Json, response::{IntoResponse, Response}
};

use serde::Deserialize;

use crate::{state::entities::{Device, PublicKey}, SharedBitacora, storage::storage::FullStorage, web3::traits::Timestamper};

use super::errors::ErrorResponse;

#[derive(Deserialize)]
pub struct POSTDeviceRequest {
    pk: String
}

pub enum POSTDeviceRequestError {
    FailedPKDecoding
}

impl TryFrom<POSTDeviceRequest> for Device {

    type Error = POSTDeviceRequestError;

    fn try_from(value: POSTDeviceRequest) -> Result<Self, Self::Error> {
        let pk: PublicKey = match value.pk.try_into() {
            Ok(pk) => pk,
            Err(_) => return Err(Self::Error::FailedPKDecoding)
        };
        Ok(Device::from(pk))
    }
}

pub async fn handler<S: FullStorage, T: Timestamper>(
    State(state): State<SharedBitacora<S, T>>,
    Json(payload): Json<POSTDeviceRequest>
) -> Response {
    let mut device = match Device::try_from(payload) {
        Ok(device) => device,
        Err(error) => match error {
            POSTDeviceRequestError::FailedPKDecoding => return ErrorResponse::bad_input("pk", Some("Failed to decode")).into_response()
        }
    };
    match state.new_device(&mut device).await {
        Ok(()) => (StatusCode::CREATED, Json(device)).into_response(),
        Err(error) => ErrorResponse::from(error).into_response()
    }
}