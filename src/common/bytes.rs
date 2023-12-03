use std::fmt::Display;

use hex::FromHexError;
use serde::{Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize)]
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

impl From<Bytes32> for [u8; 32] {
    fn from(value: Bytes32) -> Self {
        value.0
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

impl Display for Bytes32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}