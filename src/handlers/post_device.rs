use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde::Deserialize;

use tracing::{error, warn};

use crate::Conf;
use crate::{
    state::entities::{Device, PublicKey},
    storage::storage::FullStorage,
    web3::traits::Timestamper,
    SharedBitacora,
};

use super::errors::ErrorResponse;

#[derive(Clone, Deserialize)]
pub struct POSTDeviceRequest {
    pk: String,
    dataset_limit: Option<u32>,
}

pub enum POSTDeviceRequestError {
    FailedPKDecoding,
}

impl TryFrom<POSTDeviceRequest> for Device {
    type Error = POSTDeviceRequestError;

    fn try_from(value: POSTDeviceRequest) -> Result<Self, Self::Error> {
        let pk: PublicKey = match value.pk.try_into() {
            Ok(pk) => pk,
            Err(_) => return Err(Self::Error::FailedPKDecoding),
        };
        Ok(Device::from(pk))
    }
}

pub async fn handler<S: FullStorage, T: Timestamper>(
    State(state): State<SharedBitacora<S, T>>,
    Json(payload): Json<POSTDeviceRequest>,
) -> Response {
    let mut device = match Device::try_from(payload.clone()) {
        Ok(device) => device,
        Err(error) => match error {
            POSTDeviceRequestError::FailedPKDecoding => {
                warn!(pk=payload.pk, "Failed to decode input public key");
                return ErrorResponse::bad_input("pk", Some("Failed to decode")).into_response()
            }
        },
    };
    let dataset_limit = match payload.dataset_limit {
        Some(limit) => limit,
        None => Conf::get_default_dataset_limit(),
    };
    match state.new_device(&mut device, dataset_limit).await {
        Ok(()) => (StatusCode::CREATED, Json(device)).into_response(),
        Err(error) => {
            error!(device_id=device.id, "{}", error);
            ErrorResponse::from(error).into_response()
        }
    }
}
