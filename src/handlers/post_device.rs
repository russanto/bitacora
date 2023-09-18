use axum::{
    http::StatusCode,
    extract::State,
    Json, response::{IntoResponse, Response}
};

use serde::Deserialize;

use crate::{state::entities::{Device, PublicKey}, SharedBitacora};

use super::errors::ErrorResponse;

#[derive(Deserialize)]
pub struct POSTDeviceRequest {
    pk: String
}

impl From<POSTDeviceRequest> for Device {
    fn from(value: POSTDeviceRequest) -> Self {
        Device::from(PublicKey(value.pk))
    }
}

pub async fn handler(
    State(state): State<SharedBitacora>,
    Json(payload): Json<POSTDeviceRequest>
) -> Response {
    let mut device = Device::from(payload);
    match state.new_device(&mut device) {
        Ok(()) => (StatusCode::CREATED, Json(device)).into_response(),
        Err(error) => ErrorResponse::from(error).into_response()
    }
}