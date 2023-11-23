use serde::{Deserialize, Serialize};
use sha2::{ Digest, Sha256 };

use crate::web3::traits::Web3Info;

pub const ID_BYTE_LENGTH: u8 = 16;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicKey(pub String);

impl From<[u8; 32]> for PublicKey {
    fn from(value: [u8; 32]) -> Self {
        let s: String = value.iter().map(|byte| format!("{:02x}", byte)).collect();
        PublicKey(s)
    }
}

impl From<PublicKey> for [u8; 32] {
    fn from(value: PublicKey) -> Self {
        let bytes = hex::decode(value.0).map_err(|_| "Invalid Hex String").unwrap();

        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        array
    }
}

pub type DeviceId = String;

#[derive(Clone, Debug, Serialize)]
pub struct Device {
    pub id: DeviceId,
    pub pk: PublicKey,
    pub web3: Option<Web3Info>
}

impl From<PublicKey> for Device {
    fn from(value: PublicKey) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(value.0.as_bytes());
        Device {
            id: bs58::encode(hasher.finalize()).into_string(),
            pk: value.clone(),
            web3: None
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct LocalizationPoint {
    pub longitude: f64,
    pub latitude: f64,
}

pub type FlightDataId = String;

#[derive(Clone, Debug, Serialize)]
pub struct FlightData {
    pub id: FlightDataId,
    pub signature: String,
    pub timestamp: u64,
    pub localization: LocalizationPoint,
    pub payload: String
}

impl FlightData {
    pub fn id(timestamp: u64, device_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_be_bytes());
        hasher.update(device_id);
        bs58::encode(hasher.finalize()).into_string()
    }
}

pub type DatasetId = String;

pub type MerkleRoot = [u8; 32];

#[derive(Clone, Debug, Serialize)]
pub struct MerkleTree {
    pub root: MerkleRoot
}

#[derive(Clone, Debug, Serialize)]
pub struct Dataset {
    pub id: DatasetId,
    pub limit: u32,
    pub count: u32,
    pub merkle_tree: Option<MerkleTree>,
    pub web3: Option<Web3Info>
}