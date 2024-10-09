use alloy::contract::{CallBuilder, CallDecoder};
use alloy::network::{EthereumWallet, Network};
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::sol;
use alloy::transports::Transport;

use async_trait::async_trait;

use tokio::sync::mpsc;
use tokio::task;

use tracing::info;

use crate::common::prelude::*;
use crate::state::entities::{Dataset, Device, DeviceId, FlightData};

use super::traits::{
    Blockchain, MerkleTreeReceipt, Timestamper, Tx, TxHash, TxStatus, Web3Error, Web3Info,
    Web3Result,
};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    BitacoraContract,
    "contracts/artifacts/contracts/Bitacora.sol/Bitacora.json"
);

#[derive(Debug)]
pub enum TimestamperError {
    ProviderError,
    ContractError,
}

pub type TimestamperResult<T> = std::result::Result<T, Web3Error>;

struct EVMTimestamperEnvelope {
    operation: EVMTimestamperOperation,
    response: mpsc::Sender<TimestamperResult<TxHash>>,
}

enum EVMTimestamperOperation {
    RegisterDevice(Device),
    RegisterDataset(Dataset, DeviceId, Bytes32),
}

pub struct EVMTimestamper {
    sender: mpsc::UnboundedSender<EVMTimestamperEnvelope>,
    at: Address,
}

impl EVMTimestamper {
    pub async fn initialize_contract(
        url: String,
        wallet: EthereumWallet,
    ) -> TimestamperResult<Address> {
        let provider = match ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_builtin(&url)
            .await
        {
            Ok(provider) => provider,
            Err(_) => return Err(Web3Error::ProviderConnectionFailed),
        };
        let contract = BitacoraContract::deploy(provider).await.unwrap();
        Ok(contract.address().clone())
    }

    pub async fn new(
        url: String,
        at: Address,
        wallet: EthereumWallet,
    ) -> TimestamperResult<EVMTimestamper> {
        let (sender, mut receiver) = mpsc::unbounded_channel::<EVMTimestamperEnvelope>();
        let provider = match ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_builtin(&url)
            .await
        {
            Ok(provider) => provider,
            Err(_) => return Err(Web3Error::ProviderConnectionFailed),
        };
        let contract = BitacoraContract::new(at, provider);
        task::spawn(async move {
            while let Some(envelope) = receiver.recv().await {
                match envelope.operation {
                    EVMTimestamperOperation::RegisterDevice(device) => {
                        let tx = contract.registerDevice(device.id.clone(), device.pk.try_into().unwrap());
                        EVMTimestamper::handle_tx(tx, &envelope.response).await;
                    }
                    EVMTimestamperOperation::RegisterDataset(dataset, device_id, merkle_root) => {
                        let tx = contract.registerDataset(
                            dataset.id.clone(),
                            device_id,
                            merkle_root.0.into(),
                        );
                        EVMTimestamper::handle_tx(tx, &envelope.response).await;
                    }
                };
            }
        });
        Ok(EVMTimestamper { sender, at })
    }

    pub fn address(&self) -> Address {
        self.at.clone()
    }

    async fn handle_tx<T, P, D, N>(
        call: CallBuilder<T, P, D, N>,
        response: &mpsc::Sender<TimestamperResult<TxHash>>,
    ) where
        T: Transport + Send + Sync + Clone,
        P: Provider<T, N>,
        D: CallDecoder,
        N: Network,
    {
        match call.send().await {
            Err(_) => {
                // If there is no one waiting there is nothing more to handle
                let _ = response.send(Err(Web3Error::SubmissionFailed)).await;
            }
            Ok(tx) => {
                match tx.watch().await {
                    Err(_) => {
                        // If there is no one waiting there is nothing more to handle
                        let _ = response.send(Err(Web3Error::SubmissionFailed)).await;
                    }
                    Ok(tx_hash) => {
                        // If there is no one waiting there is nothing more to handle
                        let _ = response.send(Ok(tx_hash.0.into())).await;
                        info!(tx_hash = tx_hash.to_string(), "Transaction confirmed");
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Timestamper for EVMTimestamper {
    type MerkleTree = MerkleTreeOZ;

    async fn register_device(&self, device: &Device) -> Web3Result {
        let (response, mut receiver) = mpsc::channel::<TimestamperResult<TxHash>>(1);
        if self
            .sender
            .send(EVMTimestamperEnvelope {
                operation: EVMTimestamperOperation::RegisterDevice(device.clone()),
                response,
            })
            .is_err()
        {
            return Err(Web3Error::InternalError(String::from(
                "Internal EVMTimestamper queue is unavailable ",
            )));
        }
        match receiver.recv().await {
            Some(result) => match result {
                Ok(tx_hash) => Ok(Web3Info {
                    blockchain: Blockchain::ethereum(),
                    tx: Tx {
                        hash: tx_hash,
                        status: TxStatus::Confirmed,
                    },
                    merkle_receipt: None,
                }),
                Err(w3_err) => Err(w3_err),
            },
            None => Err(Web3Error::InternalError(String::from(
                "Internal EVMTimestamper response was lost",
            ))),
        }
    }

    async fn register_dataset(
        &self,
        dataset: &Dataset,
        device_id: &String,
        flight_datas: &[FlightData],
    ) -> Web3Result {
        let mut fd_mt = MerkleTreeOZ::new();
        for fd in flight_datas {
            fd_mt.append(&fd.to_bytes());
        }
        let merkle_root = fd_mt.root().unwrap();
        let (response, mut receiver) = mpsc::channel::<TimestamperResult<TxHash>>(1);
        if self
            .sender
            .send(EVMTimestamperEnvelope {
                operation: EVMTimestamperOperation::RegisterDataset(
                    dataset.clone(),
                    device_id.clone(),
                    merkle_root,
                ),
                response,
            })
            .is_err()
        {
            return Err(Web3Error::InternalError(String::from(
                "Internal EVMTimestamper queue is unavailable ",
            )));
        }
        match receiver.recv().await {
            Some(result) => match result {
                Ok(tx_hash) => Ok(Web3Info {
                    blockchain: Blockchain::ethereum(),
                    tx: Tx {
                        hash: tx_hash,
                        status: TxStatus::Confirmed,
                    },
                    merkle_receipt: Some(MerkleTreeReceipt::Tree(fd_mt)),
                }),
                Err(w3_err) => Err(w3_err),
            },
            None => Err(Web3Error::InternalError(String::from(
                "Internal EVMTimestamper response was lost ",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {

    use alloy::{network::EthereumWallet, node_bindings::Anvil, signers::local::PrivateKeySigner};

    use super::*;

    #[tokio::test]
    async fn test_register_flow() {
        let anvil = Anvil::new().try_spawn().unwrap();

        // Set up signer from the first default Anvil account (Alice).
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet = EthereumWallet::from(signer);

        // Create a provider with the wallet.
        let rpc_url = anvil.endpoint();

        let address = EVMTimestamper::initialize_contract(rpc_url.clone(), wallet.clone())
            .await
            .unwrap();
        println!("Contract address: {:?}", address);

        let timestamper = EVMTimestamper::new(rpc_url, address, wallet).await.unwrap();
        let device = Device::test_instance();
        timestamper.register_device(&device).await.unwrap();

        let mut dataset = Dataset::test_instance();
        dataset.count = dataset.limit;
        let fds = FlightData::test_instance_list(dataset.limit, &device.id);

        timestamper
            .register_dataset(&dataset, &device.id, &fds)
            .await
            .unwrap();
    }
}
