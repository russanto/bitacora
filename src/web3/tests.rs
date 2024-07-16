#[cfg(test)]
mod tests {

    use std::{path::Path, sync::Arc, time::Duration};

    use crate::{
        state::entities::{Dataset, Device, FlightData, PublicKey},
        web3::{
            stub::EthereumStub,
            traits::{Blockchain, MerkleTreeOZReceipt, Timestamper, Tx, TxStatus, Web3Info},
        },
    };

    use crate::common::prelude::*;

    impl Tx {
        pub fn test_instance_confirmed() -> Tx {
            Tx {
                hash: Bytes32::random(),
                status: TxStatus::Confirmed,
            }
        }
    }

    impl Web3Info {
        pub fn test_instance_no_merkle() -> Web3Info {
            Web3Info {
                blockchain: Blockchain::ethereum(),
                tx: Tx::test_instance_confirmed(),
                merkle_receipt: None,
            }
        }
    }
}
