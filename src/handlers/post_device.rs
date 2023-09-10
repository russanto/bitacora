use axum::{
    http::StatusCode,
    extract::State,
    Json, response::{IntoResponse, Response}
};

use serde::{ Deserialize, Serialize };

use crate::state::entities::{Device, PublicKey};
use crate::storage::storage::DeviceStorage;

use super::errors::ErrorResponse;

#[derive(Deserialize)]
pub struct POSTDeviceRequest {
    pk: String
}

#[derive(Serialize)]
pub struct POSTDeviceResponse {
    id: String
}

impl From<POSTDeviceRequest> for Device {
    fn from(value: POSTDeviceRequest) -> Self {
        Device::from(PublicKey(value.pk))
    }
}

pub async fn handler<S: DeviceStorage>(
    State(state): State<S>,
    Json(payload): Json<POSTDeviceRequest>
) -> Response {
    let device = Device::from(payload);
    match state.set_device(&device) {
        Ok(already_present) => match already_present {
            true => ErrorResponse::already_exists().into_response(),
            false => (StatusCode::CREATED, Json(POSTDeviceResponse{ id: device.id })).into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response()
    }
}