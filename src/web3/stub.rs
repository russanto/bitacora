use async_trait::async_trait;
use hex;
use sha2::{Digest, Sha256};

use crate::{
    common::prelude::*,
    state::entities::{Dataset, Device, FlightData},
};

use super::traits::{
    Blockchain, MerkleTreeOZReceipt, Timestamper, Tx, TxHash, TxStatus, Web3Error, Web3Info,
};

#[derive(Default)]
pub struct EthereumStub {}

impl EthereumStub {
    pub fn get_random_tx_hash() -> TxHash {
        let mut hasher = Sha256::new();
        hasher.update(rand::random::<u64>().to_be_bytes());
        hasher.update(rand::random::<u64>().to_be_bytes());
        hex::encode(hasher.finalize()).try_into().unwrap()
    }
}

#[async_trait]
impl Timestamper for EthereumStub {
    type MerkleTree = MerkleTreeOZ;

    async fn register_dataset(
        &self,
        _dataset: &Dataset,
        _device_id: &String,
        _flight_datas: &[FlightData],
    ) -> Result<Web3Info, Web3Error> {
        Ok(Web3Info::new_with_merkle(
            Blockchain::ethereum(),
            Tx::new(EthereumStub::get_random_tx_hash(), TxStatus::Confirmed),
            MerkleTreeOZReceipt::Root(EthereumStub::get_random_tx_hash()),
        ))
    }

    async fn register_device(&self, _device: &Device) -> Result<Web3Info, Web3Error> {
        Ok(Web3Info::new(
            Blockchain::ethereum(),
            Tx::new(EthereumStub::get_random_tx_hash(), TxStatus::Confirmed),
        ))
    }

    async fn update_web3(&self, web3info: &Web3Info) -> Result<Web3Info, Web3Error> {
        let mut updated_web3 = web3info.clone();
        updated_web3.tx.status = TxStatus::Confirmed;
        Ok(updated_web3)
    }
}
