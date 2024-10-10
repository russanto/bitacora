use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use tracing::{error, info};

use super::errors::ErrorResponse;
use crate::SharedBitacora;
use crate::{
    storage::storage::{FlightDataStorage, FullStorage},
    web3::traits::Timestamper,
};

pub async fn handler<S: FullStorage, T: Timestamper>(
    Path(id): Path<String>,
    State(state): State<SharedBitacora<S, T>>,
) -> Response {
    info!(dataset_id = id, "GET /dataset/{}", id);
    match state.get_dataset(&id) {
        Ok(dataset) => {
            let mut response = (StatusCode::OK, Json(dataset)).into_response();
            response
                .headers_mut()
                .insert("Access-Control-Allow-Origin", "*".parse().unwrap());
            response
        }
        Err(error) => {
            error!(dataset_id = id, "Error getting dataset {}", error);
            let mut response = ErrorResponse::from(error).into_response();
            response
                .headers_mut()
                .insert("Access-Control-Allow-Origin", "*".parse().unwrap());
            response
        }
    }
}
