use axum::{
    extract::State,
    Json, response::{IntoResponse, Response}
};

use base64::{DecodeError, Engine as _, engine::general_purpose::STANDARD};
use serde::{ Deserialize, Serialize };

use crate::{ SharedBitacora, state::{errors::BitacoraError, entities::{FlightDataId, DatasetId}}, storage::storage::{FullStorage, DatasetStorage}, web3::traits::{Timestamper, MerkleTreeOZReceipt}, common::{prelude::*, merkle}};
use crate::state::entities::{ FlightData, LocalizationPoint };
use crate::web3::traits::Web3Info;

use super::errors::ErrorResponse;

#[derive(Debug, Deserialize)]
pub struct VerifyFlightDataRequest {
    dataset_id: DatasetId,
    flight_data: FlightData,
    proof: <MerkleTreeOZ as MerkleTree>::Proof
}

#[derive(Serialize)]
pub struct VerifyFlightDataResponse {
    pub result: bool,
    pub web3: Web3Info
}

pub async fn handler<S: FullStorage, T: Timestamper>(
    State(state): State<SharedBitacora<S, T>>,
    Json(payload): Json<VerifyFlightDataRequest>
) -> Response {
    let dataset = match state.get_dataset(&payload.dataset_id) {
        Ok(maybe_dataset) => match maybe_dataset {
            Some(dataset) => dataset,
            None => return  ErrorResponse::not_found("Dataset").into_response()
        },
        Err(_) => return ErrorResponse::storage_error().into_response()
    };
    let web3info = match dataset.web3 {
        Some(web3info) => web3info,
        None => return ErrorResponse::not_found("Dataset Web3Info").into_response()
    };
    let merkle_root = match web3info.merkle_receipt {
        Some(ref merkle_receipt) => match merkle_receipt {
            MerkleTreeOZReceipt::Root(root) => root,
            _ => return ErrorResponse::web3_error().into_response()
        },
        None => return ErrorResponse::not_found("Dataset Merkle Root").into_response()
    };
    let fd_bytes = payload.flight_data.to_bytes();
    Json(VerifyFlightDataResponse {
        result: MerkleTreeOZ::verify_from_root(&merkle_root, &fd_bytes, &payload.proof),
        web3: web3info.clone()
    }).into_response()
}

