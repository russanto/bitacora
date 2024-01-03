use async_trait::async_trait;
use ethers::types::H256;
use serde::Serialize;

use crate::common::prelude::*;
use crate::state::entities::{Device, Dataset, FlightData};

#[derive(Debug)]
pub enum Web3Error {
    ProviderConnectionFailed,
    SubmissionFailed,
    BadInputData(String)
} 

#[async_trait]
pub trait Timestamper {

    type MerkleTree: MerkleTree;

    async fn register_device(&self, device: &Device) -> Result<Web3Info, Web3Error> ;
    async fn register_dataset(&self, dataset: &Dataset, device_id: &String, flight_datas: &[FlightData]) -> Result<Web3Info, Web3Error>;
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
    #[serde(serialize_with = "serialize_as_hex")]
    pub hash: TxHash,
    pub status: TxStatus
}

impl Tx {
    pub fn new(hash: TxHash, status: TxStatus) -> Self {
        Tx { hash, status }
    }
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
pub enum MerkleTreeReceipt<MT: MerkleTree> {
    Root(MT::Root),
    Proof(MT::Proof)
}

pub type MerkleTreeOpenZeppelinReceipt = MerkleTreeReceipt<MerkleTreeOpenZepplin>;

#[derive(Clone, Debug, Serialize)]
pub struct Web3Info {
    pub blockchain: Blockchain,
    pub tx: Tx,
    pub merkle_receipt: Option<MerkleTreeOpenZeppelinReceipt>
}


impl Web3Info {
    pub fn new(blockchain: Blockchain, tx: Tx) -> Self {
        Web3Info { blockchain, tx, merkle_receipt: None }
    }

    pub fn new_with_merkle(blockchain: Blockchain, tx: Tx, merkle: MerkleTreeOpenZeppelinReceipt) -> Self {
        Web3Info { blockchain, tx, merkle_receipt: Some(merkle) }
    }
}