use std::{path::Path, sync::Arc, time::Duration};

use async_trait::async_trait;
use ethers::signers::Signer;
use ethers::prelude::JsonRpcClient;
use ethers::{
    contract::{ abigen, ContractFactory },
    core::{
        rand::thread_rng,
        types::Address,
        k256::ecdsa::SigningKey,
        utils::Anvil,
    },
    middleware::{ Middleware, SignerMiddleware },
    providers::{Http, Provider},
    signers::{ LocalWallet, Wallet },
    solc::Solc,
    utils::AnvilInstance
};

use crate::common::merkle::Keccak256;
use crate::configuration::BitacoraConfiguration;
use crate::state::entities::{Dataset, Device, PublicKey, FlightData};
use crate::web3::traits::TxStatus;
use super::traits::{MerkleTreeOpenZeppelinReceipt, Timestamper, Web3Error, Web3Info, Blockchain, Tx };

use crate::common::prelude::*;

abigen!(EthBitacoraContract, "./eth_bitacora.abi");

pub enum EthereumTimestamperState {
    Initialized,
    Ready
}

pub struct EthereumTimestamper<M: Middleware, P: JsonRpcClient> {
    provider: Arc<Provider<P>>,
    contract: EthBitacoraContract<M>,
    pub status: EthereumTimestamperState
}

fn generate_bitacora_contract_info() -> (ethers::abi::Abi, ethers::types::Bytes) {

    let contract_base_path = BitacoraConfiguration::get_web3_contract_base_dir();
    let contract_path = Path::new(&contract_base_path);
    let compiled = Solc::default().compile_source(contract_path).unwrap();
    let (abi, bytecode, _runtime_bytecode) = compiled.find("Bitacora").expect("could not find contract").into_parts_or_default();
    (abi, bytecode)
}

pub fn new_ethereum_timestamper<M: Middleware, P: JsonRpcClient>(middleware: M, provider: Provider<P>, address: &str) -> Result<EthereumTimestamper<M, P>, Web3Error> {
    let client = Arc::new(middleware);
    let provider = Arc::new(provider);

    let address: Address = match address.parse() {
        Ok(address) => address,
        Err(err) =>  {
            println!("{}", err);
            return Err(Web3Error::BadInputData(String::from("Address")))
        }
    };
    let contract = EthBitacoraContract::new(address, client);

    Ok(EthereumTimestamper {
        provider,
        contract,
        status: EthereumTimestamperState::Ready,
    })
}

pub fn new_ethereum_timestamper_from_http(endpoint: &str, address: &str) -> Result<EthereumTimestamper<Provider<Http>, Http>, Web3Error> {
    let provider = match Provider::<Http>::try_from(endpoint) {
        Ok(provider) => provider.interval(Duration::from_millis(100u64)),
        Err(_) => return Err(Web3Error::ProviderConnectionFailed)
    };
    let client = Arc::new(provider);

    let address: Address = match address.parse() {
        Ok(address) => address,
        Err(err) =>  {
            println!("{}", err);
            return Err(Web3Error::BadInputData(String::from("Address")))
        }
    };
    let contract = EthBitacoraContract::new(address, client.clone());

    Ok(EthereumTimestamper {
        provider: client.clone(),
        contract,
        status: EthereumTimestamperState::Ready,
    })
}

pub async fn new_ethereum_timestamper_from_devnode() -> (EthereumTimestamper<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, Http>, AnvilInstance) {
    // 1. Start dev node and configure contract paths
    let anvil = Anvil::new().spawn();

    let (abi, bytecode) = generate_bitacora_contract_info();

    // 2. instantiate wallet
    let wallet: LocalWallet = anvil.keys()[0].clone().into();

    // 3. connect to the network
    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap().interval(Duration::from_millis(10u64));

    // 4. instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(anvil.chain_id()));
    let client = Arc::new(client);

    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi, ethers::types::Bytes(bytecode.0), client.clone());

    // 6. deploy it with the constructor arguments
    let contract = factory.deploy(()).unwrap().send().await.unwrap();

    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap().interval(Duration::from_millis(10u64));
    let timestamper = new_ethereum_timestamper(
        client,
        provider,
        hex::encode(contract.address()).as_str(),
    );
    if timestamper.is_err() {
        panic!("Error creating timestamp");
    }
    (timestamper.unwrap(), anvil)
}

pub async fn new_ethereum_timestamper_from_url_with_sk(url: &str, sk: &str) -> Result<EthereumTimestamper<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, Http>, Web3Error> {

    let (abi, bytecode) = generate_bitacora_contract_info();
    
    // 2. instantiate wallet
    let wallet: LocalWallet = sk.parse::<LocalWallet>().unwrap();
    // 3. connect to the network
    let provider = Provider::<Http>::try_from(url).unwrap();
    let chain_id = match provider.get_chainid().await {
        Ok(chain_id) => chain_id,
        Err(_) => return Err(Web3Error::ProviderConnectionFailed)
    };

    // 4. instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(chain_id.as_u64()));
    let client = Arc::new(client);

    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(abi, ethers::types::Bytes(bytecode.0), client.clone());

    // 6. deploy it with the constructor arguments
    let contract = factory.deploy(()).unwrap().send().await.unwrap();

    let provider = Provider::<Http>::try_from(url).unwrap().interval(Duration::from_millis(10u64));
    let timestamper = new_ethereum_timestamper(
        client,
        provider,
        hex::encode(contract.address()).as_str(),
    );
    if timestamper.is_err() {
        panic!("Error creating timestamp");
    }
    Ok(timestamper.unwrap())
}

impl <M: ethers::providers::Middleware + 'static, P: JsonRpcClient> EthereumTimestamper<M, P>{
    pub async fn get_device(&self, id: String) -> Result<Device, Box<dyn std::error::Error>> {
        let device_response = self.contract.devices(String::from(id));
        let result = device_response.call().await?;
        Ok(Device { id: result.0, pk: PublicKey::from(result.1), web3: Option::None })
    }

    // pub async fn get_dataset(&self, id: String, device_id: String) -> Result<<<EthereumTimestamper<M, P> as Timestamper>::MerkleTree as MerkleTree>::Root, Box<dyn std::error::Error>> {
    //     let dataset_response = self.contract.get_dataset(id, device_id);
    //     let result = dataset_response.call().await?;
    //     Ok(Bytes32(result))
    // }
}

#[async_trait]
impl <M: ethers::providers::Middleware + 'static, P: JsonRpcClient> Timestamper for EthereumTimestamper<M, P> {

    type MerkleTree = MerkleTreeOpenZepplin;

    async fn register_device(&self, device: &Device) -> Result<Web3Info, Web3Error>  {
        let device_response = self.contract.register_device(device.id.clone(), device.pk.0);
        
        let x = match device_response.send().await {
            Ok(pending_tx) => {
                let maybe_receipt = pending_tx.await;
                match maybe_receipt {
                    Ok(Some(receipt)) => Ok(Web3Info::new(
                        Blockchain::devnet(),
                        Tx::new(
                            receipt.transaction_hash.try_into().unwrap(),
                            TxStatus::Confirmed
                        )   
                    )),
                    Ok(None) => unimplemented!(),
                    Err(_) => unimplemented!()
                }
                
            },
            Err(error) => {
                println!("{:?}", error);
                Err(Web3Error::SubmissionFailed)
            }
        };
        x
    }

    async fn register_dataset(&self, dataset: &Dataset, device_id: &String, flight_datas: &[FlightData]) -> Result<Web3Info, Web3Error> {
        let mut fd_mt = MerkleTreeOpenZepplin::new();
        for fd in flight_datas {
            fd_mt.append(&fd.to_bytes());
        }
        let merkle_root = (&fd_mt as &dyn MerkleTree<FlightData, Node = Bytes32>).root().unwrap();
        let response = self.contract.register_dataset(
            dataset.id.clone(),
            device_id.clone(),
            merkle_root.into()
        );
        
        let x = match response.send().await {
            Ok(pending_tx) => {
                let maybe_receipt = pending_tx.await;
                match maybe_receipt {
                    Ok(Some(receipt)) => Ok(Web3Info::new_with_merkle(
                        Blockchain::devnet(),
                        Tx::new(
                            receipt.transaction_hash.try_into().unwrap(),
                            TxStatus::Confirmed
                        ),
                        MerkleTreeOpenZeppelinReceipt::Tree(fd_mt)
                    )),
                    Ok(None) => unimplemented!(),
                    Err(_) => unimplemented!()
                }
                
            },
            Err(error) => {
                println!("{:?}", error);
                Err(Web3Error::SubmissionFailed)
            }
        };
        x
    }

    async fn update_web3(&self, web3info: &Web3Info) -> Result<Web3Info, Web3Error> {
        let x = match self.provider.get_transaction(web3info.tx.hash.clone()).await {
            Ok(maybe_tx) => match maybe_tx {
                Some(tx) => {
                    println!("{:?}", tx);
                    web3info
                },
                None => return Err(Web3Error::ProviderConnectionFailed)
            }
            Err(_) => return Err(Web3Error::ProviderConnectionFailed)
        };
        Ok(x.clone())
    }
}