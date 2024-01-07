use std::{convert::TryInto, fmt::Debug, fmt::Display};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use hex::FromHexError;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::de::{self, Unexpected, Visitor};
use sha2::digest::{generic_array::GenericArray, typenum::U32};
use std::fmt;

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd)]
pub struct Bytes32(pub [u8; 32]);

impl Bytes32 {
    pub fn to_string(&self) -> String {
        String::from(self)
    }
}

impl Serialize for Bytes32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_as_hex(self, serializer)
    }
}

struct Bytes32Visitor;

impl<'de> Visitor<'de> for Bytes32Visitor {
    type Value = Bytes32;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string starting with 0x followed by 64 hexadecimal characters")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value.starts_with("0x") && value.len() == 66 {
            let bytes = match hex::decode(&value[2..]) {
                Ok(bytes) => bytes,
                Err(_) => return Err(E::custom("invalid hexadecimal")),
            };
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Ok(Bytes32(arr))
        } else {
            Err(E::custom("string does not start with 0x or has an incorrect length"))
        }
    }
}

impl<'de> Deserialize<'de> for Bytes32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Bytes32Visitor)
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

impl From<GenericArray<u8, U32>> for Bytes32 {
    fn from(value: GenericArray<u8, U32>) -> Self {
        let mut result = Bytes32::default();
        result.0.copy_from_slice(value.as_slice());
        result
    }
}

#[derive(Debug)]
pub struct BadArrayLength(usize);

impl TryFrom<Vec<u8>> for Bytes32 {

    type Error = BadArrayLength;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok(r) => Ok(Bytes32(r)),
            Err(err) => Err(BadArrayLength(err.len()))
        }
    }
}

impl Default for Bytes32 {
    fn default() -> Self {
        Bytes32([0u8; 32])
    }
}

impl Display for Bytes32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl Debug for Bytes32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

pub fn serialize_as_hex<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>, // Ensure T can be referenced as a byte slice
{
    let hex_string = format!("0x{}", hex::encode(value.as_ref()));
    serializer.serialize_str(&hex_string)
}

pub fn serialize_b64<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>, // Ensure T can be referenced as a byte slice
{
    let hex_string = STANDARD.encode(value.as_ref());
    serializer.serialize_str(&hex_string)
}

pub fn deserialize_b64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Base64Visitor;

    impl<'de> Visitor<'de> for Base64Visitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a Base64 encoded string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            STANDARD.decode(value).map_err(|_err| {
                E::invalid_value(Unexpected::Str(value), &self)
            })
        }
    }

    deserializer.deserialize_str(Base64Visitor)
}