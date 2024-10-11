use std::fmt;
use std::{convert::TryInto, fmt::Debug, fmt::Display};

use alloy::primitives::FixedBytes;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use hex::FromHexError;
use rand::Rng;
use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::digest::{generic_array::GenericArray, typenum::U32};

use crate::state::errors::BitacoraError;

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd)]
pub struct Bytes<const SIZE: usize>(pub [u8; SIZE]);

pub type Bytes32 = Bytes<32>;
pub type Bytes64 = Bytes<64>;

impl<const SIZE: usize> Bytes<SIZE> {
    // Do not use for crypto purpose
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut arr = [0u8; SIZE];
        rng.fill(&mut arr[..]);
        Self(arr)
    }

    pub fn to_string(&self) -> String {
        String::from(self)
    }
}

impl<const SIZE: usize> From<[u8; SIZE]> for Bytes<SIZE> {
    fn from(value: [u8; SIZE]) -> Self {
        Bytes::<SIZE>(value)
    }
}

impl<const SIZE: usize> Serialize for Bytes<SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_as_hex(self, serializer)
    }
}

struct BytesVisitor<const SIZE: usize>;

impl<'de, const SIZE: usize> Visitor<'de> for BytesVisitor<SIZE> {
    type Value = Bytes<SIZE>;

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
            let mut arr = [0u8; SIZE];
            arr.copy_from_slice(&bytes);
            Ok(Bytes::<SIZE>(arr))
        } else {
            Err(E::custom(
                "string does not start with 0x or has an incorrect length",
            ))
        }
    }
}

impl<'de, const SIZE: usize> Deserialize<'de> for Bytes<SIZE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BytesVisitor::<SIZE>)
    }
}

impl<const SIZE: usize> AsRef<[u8]> for Bytes<SIZE> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const SIZE: usize> From<Bytes<SIZE>> for [u8; SIZE] {
    fn from(value: Bytes<SIZE>) -> Self {
        value.0
    }
}

impl<const SIZE: usize> From<Bytes<SIZE>> for String {
    fn from(value: Bytes<SIZE>) -> Self {
        String::from(&value)
    }
}
impl<const SIZE: usize> From<&Bytes<SIZE>> for String {
    fn from(value: &Bytes<SIZE>) -> Self {
        hex::encode(value.0)
    }
}

impl<const SIZE: usize> TryFrom<&[u8]> for Bytes<SIZE> {
    type Error = BitacoraError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != SIZE as usize {
            println!("{:?}", value.len());
            return Err(BitacoraError::Web3Error); //TODO: just a placeholder, need to be changed
        }
        let mut bytes = [0u8; SIZE];
        bytes.copy_from_slice(value);
        Ok(Bytes::<SIZE>(bytes))
    }
}

impl<const SIZE: usize> TryFrom<&str> for Bytes<SIZE> {
    type Error = FromHexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = if value.starts_with("0x") {
            &value[2..]
        } else {
            value
        };
        let mut bytes = [0u8; SIZE];
        hex::decode_to_slice(value, &mut bytes)?;
        Ok(Bytes::<SIZE>(bytes))
    }
}

impl<const SIZE: usize> TryFrom<String> for Bytes<SIZE> {
    type Error = FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Bytes::<SIZE>::try_from(value.as_str())
    }
}

impl<const SIZE: usize> From<GenericArray<u8, U32>> for Bytes<SIZE> {
    fn from(value: GenericArray<u8, U32>) -> Self {
        let mut result = Bytes::<SIZE>::default();
        result.0.copy_from_slice(value.as_slice());
        result
    }
}

#[derive(Debug)]
pub enum BytesDecodeError {
    BadLength(usize),
}

impl<const SIZE: usize> TryFrom<Vec<u8>> for Bytes<SIZE> {
    type Error = BytesDecodeError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.try_into() {
            Ok(r) => Ok(Bytes::<SIZE>(r)),
            Err(err) => Err(BytesDecodeError::BadLength(err.len())),
        }
    }
}

impl<const SIZE: usize> Default for Bytes<SIZE> {
    fn default() -> Self {
        Bytes::<SIZE>([0u8; SIZE])
    }
}

impl<const SIZE: usize> Display for Bytes<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl<const SIZE: usize> Debug for Bytes<SIZE> {
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
    let b64_string = STANDARD.encode(value.as_ref());
    serializer.serialize_str(&b64_string)
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
            STANDARD
                .decode(value)
                .map_err(|_err| E::invalid_value(Unexpected::Str(value), &self))
        }
    }

    deserializer.deserialize_str(Base64Visitor)
}

pub fn deserialize_b64_to_bytes<'de, D, const SIZE: usize>(
    deserializer: D,
) -> Result<Bytes<SIZE>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Base64Visitor<const SIZE: usize>;

    impl<'de, const SIZE: usize> Visitor<'de> for Base64Visitor<SIZE> {
        type Value = Bytes<SIZE>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a Base64 encoded string representing $SIZE bytes")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            STANDARD
                .decode(value)
                .map_err(|_err| E::invalid_value(Unexpected::Str(value), &self))?
                .try_into()
                .map_err(|err| match err {
                    BytesDecodeError::BadLength(len) => E::invalid_length(len, &self),
                })
        }
    }

    deserializer.deserialize_str(Base64Visitor)
}

impl<const SIZE: usize> From<FixedBytes<SIZE>> for Bytes<SIZE> {
    fn from(value: FixedBytes<SIZE>) -> Self {
        Bytes::<SIZE>(value.0)
    }
}

impl<const SIZE: usize> TryFrom<Bytes<SIZE>> for FixedBytes<32> {
    type Error = BitacoraError;

    fn try_from(value: Bytes<SIZE>) -> Result<Self, Self::Error> {
        if value.0.len() < 32 {
            return Err(BitacoraError::Web3Error);
        }
        let mut ret = FixedBytes::<32>([0; 32]);
        ret.copy_from_slice(value.0[..32].as_ref());
        Ok(ret)
    }
}
