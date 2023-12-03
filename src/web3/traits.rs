use async_trait::async_trait;
use ethers::types::H256;
use serde::Serialize;

use crate::state::entities::{Device, Dataset};
use crate::common::bytes::Bytes32;

#[derive(Debug)]
pub enum Web3Error {
    ProviderConnectionFailed,
    SubmissionFailed,
    BadInputData(String)
} 

#[async_trait]
pub trait Timestamper {
    async fn register_device(&self, device: &Device) -> Result<Web3Info, Web3Error> ;
    async fn register_dataset(&self, dataset: &Dataset, device_id: &String) -> Result<Web3Info, Web3Error>;
    async fn update_web3(&self, web3info: &Web3Info) -> Result<Web3Info, Web3Error>;
}

#[derive(Clone, Debug, Serialize)]
pub enum TxStatus {
    Submitted,
    Included,
    Confirmed
}

pub type TxHash = Bytes32;

impl From<H256> for TxHash {
    fn from(value: H256) -> Self {
        Bytes32(value.0)
    }
}

impl From<TxHash> for H256 {
    fn from(value: Bytes32) -> Self {
        H256::from(value.0)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Tx {
    #[serde(serialize_with = "Bytes32::serialize_as_hex")]
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