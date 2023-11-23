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

    #[test]
    fn test_register_device() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (timestamper, _anvil) = rt.block_on(new_ethereum_timestamper_from_devnode());
        
        let device = Device {
            id: String::from("SomeID24"),
            pk: PublicKey(String::from("1224000000000000000000000000000000000000000000000000000000123456")),
            web3: Option::None
        };

        match timestamper.register_device(&device) {
            Err(err) => {
                println!("{:?}", err);
                panic!("Device registration failed");
            },
            Ok(_) => {
                let gotten_device = rt.block_on(timestamper.get_device(String::from("SomeID24")));
                assert!(gotten_device.is_ok(), "Could not get Device after registration");
                let gotten_device = gotten_device.unwrap();
                assert_eq!(gotten_device.id, device.id, "Registered a different ID than supplied");
                assert_eq!(gotten_device.pk.0, device.pk.0, "Registered a different Public Key than supplied");
            }
        }
    }

    #[test]
    fn test_register_dataset() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (timestamper, _anvil) = rt.block_on(new_ethereum_timestamper_from_devnode());
        
        let device = Device {
            id: String::from("SomeID24"),
            pk: PublicKey(String::from("1224000000000000000000000000000000000000000000000000000000123456")),
            web3: Option::None
        };

        let _ = timestamper.register_device(&device);

        // let dummy_merkle_root: MerkleRoot = EthereumStub::get_random_tx_hash().0;

        let dataset = Dataset {
            id: String::from("Some Id"),
            limit: 10,
            count: 10,
            merkle_tree: Some(MerkleTree { root: EthereumStub::get_random_tx_hash().0}),
            web3: None
        };

        match timestamper.register_dataset(&dataset, &device.id) {
            Err(err) => {
                println!("{:?}", err);
                panic!("Dataset registration failed");
            },
            Ok(_) => {
                let gotten_merkle_root = rt.block_on(timestamper.get_dataset(dataset.id, device.id)).unwrap();
                assert_eq!(gotten_merkle_root, dataset.merkle_tree.unwrap().root)
            }
        }
    }

}