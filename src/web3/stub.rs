use hex;
use sha2::{Digest, Sha256};

use crate::state::entities::{ Dataset, Device };

use super::traits::{ Blockchain, Timestamper, Web3Info, TxStatus, Tx, Web3Error, TxHash };

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

impl Timestamper for EthereumStub {
    fn register_dataset(&self, _dataset: &Dataset, _device_id: &String) -> Result<Web3Info, Web3Error> {
        Ok(Web3Info {
            blockchain: Blockchain::ethereum(),
            tx: Tx {
                hash: EthereumStub::get_random_tx_hash(),
                status: TxStatus::Confirmed
            }
        })
    }

    fn register_device(&self, _device: &Device) -> Result<Web3Info, Web3Error> {
        Ok(Web3Info {
            blockchain: Blockchain::ethereum(),
            tx: Tx {
                hash: EthereumStub::get_random_tx_hash(),
                status: TxStatus::Confirmed
            }
        })
    }

    fn update_web3(&self, web3info: &Web3Info) -> Result<Web3Info, Web3Error> {
        let mut updated_web3 = web3info.clone();
        updated_web3.tx.status = TxStatus::Confirmed;
        Ok(updated_web3)
    }
}