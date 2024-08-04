#[cfg(test)]
mod tests {

    use crate::web3::traits::{Blockchain, Tx, TxStatus, Web3Info};

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
