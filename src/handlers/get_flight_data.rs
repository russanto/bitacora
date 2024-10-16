use axum::{extract::{State, Path}, http::StatusCode, Json, response::{IntoResponse, Response}};

use crate::{storage::storage::{FlightDataStorage, FullStorage}, web3::traits::Timestamper, state::entities::FlightDataId};
use crate::SharedBitacora;

use super::errors::ErrorResponse;

pub async fn handler<S: FullStorage, T: Timestamper>(
    Path(id): Path<String>,
    State(state): State<SharedBitacora<S, T>>
) -> Response {
    match FlightDataId::try_from(id) {
        Ok(f_id) => match state.get_flight_data(&f_id) {
            Ok(query_result) => {
                match query_result {
                    Some(fd) => (StatusCode::OK, Json(fd)).into_response(),
                    None => ErrorResponse::not_found("FlightData").into_response()
                }
            },
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(())).into_response()
        },
        Err(_) => ErrorResponse::bad_input("id", Some("Can't decode Id")).into_response()
    }
}