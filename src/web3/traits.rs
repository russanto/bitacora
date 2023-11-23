use ethers::types::H256;
use hex::FromHexError;
use serde::Serialize;

use crate::state::entities::{Device, Dataset};

#[derive(Debug)]
pub enum Web3Error {
    ProviderConnectionFailed,
    SubmissionFailed,
    BadInputData(String)
} 

pub trait Timestamper {
    fn register_device(&self, device: &Device) -> Result<Web3Info, Web3Error> ;
    fn register_dataset(&self, dataset: &Dataset, device_id: &String) -> Result<Web3Info, Web3Error>;
    fn update_web3(&self, web3info: &Web3Info) -> Result<Web3Info, Web3Error>;
}

#[derive(Clone, Debug, Serialize)]
pub enum TxStatus {
    Submitted,
    Included,
    Confirmed
}

#[derive(Clone, Debug, Serialize)]
pub struct TxHash(pub [u8; 32]);

impl TxHash {
    pub fn to_string(&self) -> String {
        String::from(self)
    }
}

impl From<H256> for TxHash {
    fn from(value: H256) -> Self {
        TxHash(value.0)
    }
}

impl From<TxHash> for String {
    fn from(value: TxHash) -> Self {
        String::from(&value)
    }
}

impl From<&TxHash> for String {
    fn from(value: &TxHash) -> Self {
        hex::encode(value.0)
    }
}

impl TryFrom<&str> for TxHash {

    type Error = FromHexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = if value.starts_with("0x") {
            &value[2..]   
        } else {
            value
        };
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(value, &mut bytes)?;
        Ok(TxHash(bytes))
    }
}

impl TryFrom<String> for TxHash {

    type Error = FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        TxHash::try_from(value.as_str())
    }
}

impl From<TxHash> for H256 {
    fn from(value: TxHash) -> Self {
        H256::from(value.0)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Tx {
    pub hash: TxHash,
    pub status: TxStatus
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum Blockchain {
    EVM { chain: String }
}

impl Blockchain {
    pub fn ethereum() -> Blockchain {
        Blockchain::EVM { chain: String::from("Ethereum") }
    }

    pub fn devnet() -> Blockchain {
        Blockchain::EVM { chain: String::from("Devnet") }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Web3Info {
    pub blockchain: Blockchain,
    pub tx: Tx
}