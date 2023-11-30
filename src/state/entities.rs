use std::fmt::Display;

use hex::FromHexError;
use serde::{Deserialize, Serialize, Serializer};
use sha2::{ Digest, Sha256 };

use crate::web3::traits::Web3Info;

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
        FlightDataId(String::from(""))
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

#[derive(Clone, Debug, Serialize)]
pub struct FlightData {
    pub id: FlightDataId,
    pub signature: String,
    pub timestamp: u64,
    pub localization: LocalizationPoint,
    pub payload: String
}

pub type DatasetId = String;

pub type MerkleRoot = Bytes32;

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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Bytes32(pub [u8; 32]);

impl Bytes32 {
    pub fn to_string(&self) -> String {
        String::from(self)
    }

    pub fn serialize_as_hex<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>, // Ensure T can be referenced as a byte slice
    {
        let hex_string = format!("0x{}", hex::encode(value.as_ref()));
        serializer.serialize_str(&hex_string)
    }
}

impl AsRef<[u8]> for Bytes32 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Bytes32> for String {
    fn from(value: Bytes32) -> Self {
        String::from(&value)
    }
}

impl From<&Bytes32> for String {
    fn from(value: &Bytes32) -> Self {
        hex::encode(value.0)
    }
}

impl TryFrom<&str> for Bytes32 {

    type Error = FromHexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = if value.starts_with("0x") {
            &value[2..]   
        } else {
            value
        };
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(value, &mut bytes)?;
        Ok(Bytes32(bytes))
    }
}

impl TryFrom<String> for Bytes32 {

    type Error = FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Bytes32::try_from(value.as_str())
    }
}