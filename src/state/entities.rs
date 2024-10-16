use std::{fmt::Display, hash};

use ethers::utils::keccak256;
use hex::FromHexError;
use serde::{Deserialize, Serialize, Serializer};
use sha2::{ Digest, Sha256 };

use crate::{web3::traits::Web3Info};

use crate::common::prelude::*;

use super::errors::BitacoraError;

pub const ID_BYTE_LENGTH: u8 = 16;
pub const FLIGHT_DATA_ID_PREFIX: u8 = 1;

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
pub struct FlightDataId(Bytes32);

impl FlightDataId {
    pub fn new(timestamp: u64, device_id: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(FLIGHT_DATA_ID_PREFIX.to_be_bytes());
        hasher.update(timestamp.to_be_bytes());
        hasher.update(device_id);
        FlightDataId(hasher.finalize().into())
    }

    pub fn to_string(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    pub fn to_hex_string(&self) -> String {
        hex::encode(&self.0)
    }
}

impl Default for FlightDataId {
    fn default() -> Self {
        FlightDataId(Bytes32::default())
    }
}

impl TryFrom<String> for FlightDataId {
    type Error = BitacoraError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match bs58::decode(value).into_vec() {
            Ok(id_bytes) => {
                if id_bytes.len() != 32 {
                    return Err(BitacoraError::BadIdFormat);
                }
                Ok(FlightDataId(id_bytes.try_into().unwrap()))
            },
            Err(_) => Err(BitacoraError::BadIdFormat)
        }
    }
}

impl From<FlightDataId> for String {
    fn from(value: FlightDataId) -> Self {
        value.to_string()
    }
}

impl AsRef<[u8]> for FlightDataId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FlightData {
    pub id: FlightDataId,
    pub signature: String,
    pub timestamp: u64,
    pub localization: LocalizationPoint,
    pub payload: Vec<u8>
}

impl FlightData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut accumulator = Vec::new();
        accumulator.extend_from_slice(self.id.as_ref());
        accumulator.extend_from_slice(self.timestamp.to_be_bytes().as_slice());
        accumulator.extend_from_slice(self.localization.latitude.to_be_bytes().as_slice());
        accumulator.extend_from_slice(self.localization.longitude.to_be_bytes().as_slice());
        accumulator.extend(&self.payload);
        accumulator
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