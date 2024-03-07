use std::fmt::Display;


use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Unexpected, Visitor};
use sha2::{ Digest, Sha256 };
use std::fmt;

use crate::common::bytes::Bytes32DecodeError;
use crate::{web3::traits::Web3Info};

use crate::common::prelude::*;

use super::errors::{BitacoraError, IdError};

pub const ID_BYTE_LENGTH: u8 = 16;
pub const FLIGHT_DATA_ID_PREFIX: u8 = 1;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Device {
    pub id: DeviceId,
    #[serde(serialize_with = "serialize_as_hex")]
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct LocalizationPoint {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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

impl Display for FlightDataId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}

impl TryFrom<String> for FlightDataId {
    type Error = BitacoraError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match bs58::decode(value).into_vec() {
            Ok(id_bytes) => {
                id_bytes.try_into().map(|bytes32| {
                    FlightDataId(bytes32)
                }).map_err(|err| {
                    match err {
                        Bytes32DecodeError::BadLength(len) => BitacoraError::BadId(IdError::Length(len, 32))
                    }
                })
            },
            Err(_) => Err(BitacoraError::BadId(IdError::Unknown))
        }
    }
}

impl TryFrom<&String> for FlightDataId {
    type Error = BitacoraError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match bs58::decode(value).into_vec() {
            Ok(id_bytes) => {
                id_bytes.try_into().map(|bytes32| {
                    FlightDataId(bytes32)
                }).map_err(|err| {
                    match err {
                        Bytes32DecodeError::BadLength(len) => BitacoraError::BadId(IdError::Length(len, 32))
                    }
                })
            },
            Err(_) => Err(BitacoraError::BadId(IdError::Unknown))
        }
    }
}

impl From<Bytes32> for FlightDataId {
    fn from(value: Bytes32) -> Self {
        FlightDataId(value)
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

pub fn deserialize_b58_to_flight_data_id<'de, D>(deserializer: D) -> Result<FlightDataId, D::Error>
where
    D: Deserializer<'de>,
{
    struct Base58Visitor;

    impl<'de> Visitor<'de> for Base58Visitor {
        type Value = FlightDataId;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a Base58 encoded string representing 32 bytes")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            FlightDataId::try_from(String::from(value)).map_err(|err| {
                match err {
                    BitacoraError::BadId(id_err) => match id_err {
                        IdError::Length(cur, exp) => E::invalid_length(cur, &self),
                        _ => unimplemented!()
                    },
                    _ => unreachable!()
                }
            })
        }
    }

    deserializer.deserialize_str(Base58Visitor)
}

pub type Timestamp = u64;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FlightData {
    #[serde(deserialize_with = "deserialize_b58_to_flight_data_id", serialize_with = "serialize_b64")]
    pub id: FlightDataId,
    pub signature: String,
    pub timestamp: Timestamp,
    pub localization: LocalizationPoint,
    #[serde(deserialize_with = "deserialize_b64", serialize_with = "serialize_b64")]
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
pub type DatasetCounter = u32;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Dataset {
    pub id: DatasetId,
    pub limit: u32,
    pub count: u32,
    pub web3: Option<Web3Info>
}

impl Dataset {
    pub fn dataset_id(device_id: &DeviceId, device_dataset_counter: DatasetCounter) -> DatasetId {
        let mut hasher = Sha256::new();
        hasher.update(device_id);
        hasher.update(device_dataset_counter.to_be_bytes());
        bs58::encode(hasher.finalize()).into_string()
    }

    pub fn new(device_id: DeviceId, device_dataset_counter: DatasetCounter, limit: u32) -> Self {
        Dataset {
            id: Self::dataset_id(&device_id, device_dataset_counter),
            limit,
            count: 0,
            web3: None
        }
    }
}