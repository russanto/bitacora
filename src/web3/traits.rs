use async_trait::async_trait;
use ethers::types::H256;
use serde::{Deserialize, Serialize};

use crate::common::prelude::*;
use crate::state::entities::{Dataset, Device, FlightData};

pub type Web3Result = Result<Web3Info, Web3Error>;

#[derive(Debug)]
pub enum Web3Error {
    ProviderConnectionFailed,
    SubmissionFailed,
    InternalServerError,
    BadInputData(String),
}

#[async_trait]
pub trait Timestamper {

    /// The MerkleTree type used by the Timestamper to handle off-chain collections
    type MerkleTree: MerkleTree;

    /// Register a new device on the blockchain and returns the respective traceability information
    /// 
    /// # Arguments
    /// 
    /// * `device` - The device to be registered
    /// 
    /// # Returns
    /// 
    /// A Web3Result containing the traceability information of the device registration
    /// 
    /// # Errors
    /// 
    /// A Web3Error describing the registration failure
    async fn register_device(&self, device: &Device) -> Web3Result;

    /// Register a new dataset on the blockchain and returns the respective traceability information
    /// 
    /// # Arguments
    /// 
    /// * `dataset` - The dataset to be registered
    /// * `device_id` - The ID of the device that owns the dataset
    /// * `flight_datas` - The flight datas that compose the dataset
    /// 
    /// # Returns
    /// 
    /// A Web3Result containing the traceability information of the dataset registration
    /// 
    /// # Errors
    /// 
    /// A Web3Error describing the registration failure
    async fn register_dataset(
        &self,
        dataset: &Dataset,
        device_id: &String,
        flight_datas: &[FlightData],
    ) -> Result<Web3Info, Web3Error>;

    /// Computes the Merkle Proof of a FlightData entry given its traceability information
    /// 
    /// # Arguments
    /// 
    /// * `fd` - The FlightData entry to compute the Merkle Proof for
    /// * `flight_datas` - The collection of FlightData entries included in the dataset
    /// * `dataset_receipt` - The traceability information of the dataset the FlightData entry belongs to
    /// 
    /// # Returns
    /// 
    /// A Web3Result containing the Merkle Proof of the FlightData entry
    /// 
    /// # Errors
    /// 
    /// A Web3Error describing the Merkle Proof computation failure
    fn flight_data_web3_info(
        fd: &FlightData,
        flight_datas: &[FlightData],
        dataset_receipt: &Web3Info,
    ) -> Web3Result {
        let mut fd_mt = MerkleTreeOZ::new();
        for f in flight_datas {
            fd_mt.append(&f.to_bytes());
        }
        let test_bytes = fd.to_bytes();
        let proof = fd_mt.proof(&test_bytes).unwrap(); //TODO: manage here
        Ok(Web3Info {
            blockchain: dataset_receipt.blockchain.clone(),
            tx: dataset_receipt.tx.clone(),
            merkle_receipt: Some(MerkleTreeOZReceipt::Proof(proof)),
        })
    }
}

/// The status of a transaction on the blockchain
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum TxStatus {
    Submitted,
    Included,
    Confirmed,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Tx {
    #[serde(serialize_with = "serialize_as_hex")]
    pub hash: TxHash,
    pub status: TxStatus,
}

impl Tx {
    pub fn new(hash: TxHash, status: TxStatus) -> Self {
        Tx { hash, status }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum Blockchain {
    EVM { chain: String },
}

impl Blockchain {
    pub fn ethereum() -> Blockchain {
        Blockchain::EVM {
            chain: String::from("Ethereum"),
        }
    }

    pub fn devnet() -> Blockchain {
        Blockchain::EVM {
            chain: String::from("Devnet"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum MerkleTreeReceipt<MT: MerkleTree> {
    Root(MT::Node),
    Proof(MT::Proof),
    Tree(MT),
}

pub type MerkleTreeOZReceipt = MerkleTreeReceipt<MerkleTreeOZ>;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Web3Info {
    pub blockchain: Blockchain,
    pub tx: Tx,
    pub merkle_receipt: Option<MerkleTreeOZReceipt>,
}

impl Web3Info {
    pub fn new(blockchain: Blockchain, tx: Tx) -> Self {
        Web3Info {
            blockchain,
            tx,
            merkle_receipt: None,
        }
    }

    pub fn new_with_merkle(blockchain: Blockchain, tx: Tx, merkle: MerkleTreeOZReceipt) -> Self {
        Web3Info {
            blockchain,
            tx,
            merkle_receipt: Some(merkle),
        }
    }
}
