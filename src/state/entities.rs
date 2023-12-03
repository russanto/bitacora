use std::{fmt::Display, hash};

use hex::FromHexError;
use serde::{Deserialize, Serialize, Serializer};
use sha2::{ Digest, Sha256 };

use crate::{web3::traits::Web3Info};

use crate::common::prelude::*;

pub const ID_BYTE_LENGTH: u8 = 16;

#[derive(Clone, Debug)]
pub enum Entity {
    Dataset,
    Device,
    FlightData
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}

impl From<Entity> for String {
    fn from(value: Entity) -> Self {
        match value {
            Entity::Dataset => String::from("Dataset"),
            Entity::Device => String::from("Device"),
            Entity::FlightData => String::from("FlightData"),
        }
    }
}

pub type PublicKey = Bytes32;

impl From<[u8; 32]> for PublicKey {
    fn from(value: [u8; 32]) -> Self {
        Bytes32(value)
    }
}

// impl From<PublicKey> for [u8; 32] {
//     fn from(value: PublicKey) -> Self {
//         let bytes = hex::decode(value.0).map_err(|_| "Invalid Hex String").unwrap();

//         let mut array = [0u8; 32];
//         array.copy_from_slice(&bytes);
//         array
//     }
// }

pub type DeviceId = String;

#[derive(Clone, Debug, Serialize)]
pub struct Device {
    pub id: DeviceId,
    #[serde(serialize_with = "Bytes32::serialize_as_hex")]
    pub pk: PublicKey,
    pub web3: Option<Web3Info>
}

impl From<PublicKey> for Device {
    fn from(value: PublicKey) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(value.0);
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct FlightDataId(pub String);

impl FlightDataId {
    pub fn new(timestamp: u64, device_id: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_be_bytes());
        hasher.update(device_id);
        FlightDataId::from(bs58::encode(hasher.finalize()).into_string())
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Default for FlightDataId {
    fn default() -> Self {
        FlightDataId(String::new())
    }
}

impl From<String> for FlightDataId {
    fn from(value: String) -> Self {
        FlightDataId(value)
    }
}

impl From<FlightDataId> for String {
    fn from(value: FlightDataId) -> Self {
        value.0
    }
}

impl AsRef<[u8]> for FlightDataId {
    fn as_ref(&self) -> &[u8] {
        let id_as_bytes = bs58::decode(&self.0).into_vec().unwrap();
        &id_as_bytes
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FlightData {
    pub id: FlightDataId,
    pub signature: String,
    pub timestamp: u64,
    pub localization: LocalizationPoint,
    pub payload: String
}

impl AsRef<[u8]> for FlightData {
    fn as_ref(&self) -> &[u8] {
        let mut accumulator = Vec::new();
        accumulator.extend_from_slice(self.id.as_ref());
        accumulator.extend_from_slice(self.timestamp.to_be_bytes().as_slice());
        accumulator.extend_from_slice(self.localization.latitude.to_be_bytes().as_slice());
        accumulator.extend_from_slice(self.localization.longitude.to_be_bytes().as_slice());
        accumulator.extend_from_slice(self.payload.as_bytes());
        &accumulator
    }
}

pub type DatasetId = String;

#[derive(Clone, Debug, Serialize)]
pub struct Dataset {
    pub id: DatasetId,
    pub limit: u32,
    pub count: u32,
    pub merkle_root: Option<MerkleRoot>,
    pub web3: Option<Web3Info>
}