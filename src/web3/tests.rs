#[cfg(test)]
mod tests {

    use std::{
        sync::Arc,
        path::Path,
        time::Duration
    };

    use ethers::{
        contract::{ ContractFactory },
        core::{
            k256::ecdsa::SigningKey,
            utils::Anvil
        },
        middleware::SignerMiddleware,
        providers::{Http, Provider, Middleware, JsonRpcClient},
        signers::{ LocalWallet, Signer, Wallet },
        solc::Solc, utils::AnvilInstance
    };

    use crate::{state::entities::{Dataset, Device, FlightData, PublicKey}, web3::{ethereum::{new_ethereum_timestamper_from_devnode, EthereumTimestamper}, stub::EthereumStub, traits::{Blockchain, MerkleTreeOZReceipt, Timestamper, Tx, TxStatus, Web3Info}}};

    use crate::common::prelude::*;

    impl Tx {
        pub fn test_instance_confirmed() -> Tx {
            Tx {
                hash: Bytes32::random(),
                status: TxStatus::Confirmed
            }
        }
    }

    impl Web3Info {
        pub fn test_instance_no_merkle() -> Web3Info {
            Web3Info {
                blockchain: Blockchain::ethereum(),
                tx: Tx::test_instance_confirmed(),
                merkle_receipt: None
            }
        }
    }

    #[tokio::test]
    async fn test_register_device() {
        let (timestamper, _anvil) = new_ethereum_timestamper_from_devnode().await;
        
        let device_pk: PublicKey = "0x1234567890123456789012345678901234567890123456789012345678901234".try_into().unwrap();
        let device = Device::from(device_pk);

        match timestamper.register_device(&device).await {
            Err(err) => {
                println!("{:?}", err);
                panic!("Device registration failed");
            },
            Ok(_) => {
                let gotten_device = timestamper.get_device(device.id.clone()).await;
                assert!(gotten_device.is_ok(), "Could not get Device after registration");
                let gotten_device = gotten_device.unwrap();
                assert_eq!(gotten_device.id, device.id, "Registered a different ID than supplied");
                assert_eq!(gotten_device.pk.0, device.pk.0, "Registered a different Public Key than supplied");
            }
        }
    }

    #[tokio::test]
    async fn test_register_dataset() {
        let (timestamper, _anvil) = new_ethereum_timestamper_from_devnode().await;
        
        let device = Device::test_instance();

        let _ = timestamper.register_device(&device).await;

        // let dummy_merkle_root: MerkleRoot = EthereumStub::get_random_tx_hash().0;

        let dataset = Dataset {
            id: String::from("Some Id"),
            limit: 10,
            count: 10,
            web3: None
        };

        let fds = FlightData::test_instance_list(10);

        match timestamper.register_dataset(&dataset, &device.id, &fds).await {
            Err(err) => {
                println!("{:?}", err);
                panic!("Dataset registration failed");
            },
            Ok(ref mut web3info) => {
                if let Some(ref mut merkle_receipt) = web3info.merkle_receipt {
                    match merkle_receipt {
                        MerkleTreeOZReceipt::Tree(ref mut mt) => {
                            if let Some(root) = mt.root() {
                                let gotten_merkle_root = timestamper.get_dataset(dataset.id, device.id).await.unwrap();
                                assert_eq!(gotten_merkle_root, root);
                            } else {
                                unreachable!()
                            }
                        },
                        _ => unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
        }
    }

}