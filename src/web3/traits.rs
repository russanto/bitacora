use serde::Serialize;

use crate::state::entities::{Device, Dataset};

pub enum Web3Error {} 

pub trait Timestamper {
    fn register_device(&self, device: &Device) -> Result<Web3Info, Web3Error> ;
    fn register_dataset(&self, dataset: &Dataset) -> Result<Web3Info, Web3Error>;
}

#[derive(Clone, Debug, Serialize)]
pub enum TxStatus {
    Submitted,
    Included,
    Confirmed
}

#[derive(Clone, Debug, Serialize)]
pub struct Tx {
    pub hash: String,
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
}

#[derive(Clone, Debug, Serialize)]
pub struct Web3Info {
    pub blockchain: Blockchain,
    pub tx: Tx
}