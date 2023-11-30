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

    use crate::{web3::{ethereum::{new_ethereum_timestamper_from_devnode, EthereumTimestamper}, traits::Timestamper, stub::EthereumStub}, state::entities::Device, state::entities::{PublicKey, Dataset, MerkleTree, MerkleRoot}};

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
        
        let device_pk: PublicKey = "0x1234567890123456789012345678901234567890123456789012345678901234".try_into().unwrap();
        let device = Device::from(device_pk);

        let _ = timestamper.register_device(&device).await;

        // let dummy_merkle_root: MerkleRoot = EthereumStub::get_random_tx_hash().0;

        let dataset = Dataset {
            id: String::from("Some Id"),
            limit: 10,
            count: 10,
            merkle_tree: Some(MerkleTree { root: EthereumStub::get_random_tx_hash()}),
            web3: None
        };

        match timestamper.register_dataset(&dataset, &device.id).await {
            Err(err) => {
                println!("{:?}", err);
                panic!("Dataset registration failed");
            },
            Ok(_) => {
                let gotten_merkle_root = timestamper.get_dataset(dataset.id, device.id).await.unwrap();
                assert_eq!(gotten_merkle_root, dataset.merkle_tree.unwrap().root)
            }
        }
    }

}