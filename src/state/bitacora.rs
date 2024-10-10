use std::sync::Arc;

use tracing::{error, info, trace};

use crate::common::prelude::*;

use crate::storage::errors::Error as StorageError;
use crate::storage::storage::{DeviceStorage, FlightDataStorage, FullStorage};
use crate::web3::traits::{MerkleTreeOZReceipt, Timestamper, Web3Info};

use super::entities::{Dataset, DatasetId, Device, DeviceId, FlightData, FlightDataId};
use super::errors::BitacoraError;

pub const DATASET_DEFAULT_LIMIT: u32 = 10;

type SharedBitacora<S, T> = Arc<Bitacora<S, T>>;

pub struct Bitacora<S, T>
where
    S: FullStorage,
    T: Timestamper,
{
    storage: S,
    timestamper: T,
}

impl<S, T> Bitacora<S, T>
where
    S: FullStorage,
    T: Timestamper,
{
    pub fn new(storage: S, timestamper: T) -> Bitacora<S, T> {
        Bitacora {
            storage,
            timestamper,
        }
    }

    pub async fn new_flight_data(
        &self,
        fd: &FlightData,
        device_id: &DeviceId,
    ) -> Result<Dataset, BitacoraError> {
        let mut dataset = self.storage.new_flight_data(fd, device_id)?;
        info!(
            device_id = device_id,
            flight_data_id = fd.id.to_string(),
            "Created new FlightData"
        );
        if dataset.count == dataset.limit {
            // TODO: refactor to let it be asynchronous
            let fds = match self.storage.get_dataset_flight_datas(&dataset.id) {
                Ok(fds) => fds,
                Err(err) => {
                    error!(dataset_id = dataset.id, "Error getting flight datas in complete dataset {}", err);
                    return Err(BitacoraError::wrap_with_completed(err.into()))
                },
            };
            match self.timestamp_dataset(&mut dataset, device_id, &fds).await {
                Err(err) => return Err(BitacoraError::wrap_with_completed(err)),
                _ => (),
            };
            info!(
                dataset_id = dataset.id,
                device_id = device_id,
                "Timestamped Dataset"
            );
            match self.storage.update_dataset_web3(&dataset) {
                Err(err) => return Err(BitacoraError::wrap_with_completed(err.into())),
                _ => (),
            };
        }
        Ok(dataset)
    }

    pub fn new_dataset(&self, limit: u32, device_id: &DeviceId) -> Result<Dataset, BitacoraError> {
        trace!(device_id = device_id, "Creating new Dataset");
        let ds = self.storage.new_dataset(limit, device_id)?;
        info!(dataset_id = ds.id, device_id = device_id, "Created Dataset");
        Ok(ds)
    }

    pub async fn new_device(
        &self,
        device: &mut Device,
        dataset_limit: u32,
    ) -> Result<(), BitacoraError> {
        self.storage.new_device(&device, dataset_limit)?;
        self.timestamp_device(device).await
    }

    async fn timestamp_device(&self, device: &mut Device) -> Result<(), BitacoraError> {
        match self.timestamper.register_device(device).await {
            Ok(web3_info) => {
                info!(
                    device = device.id,
                    tx_hash = web3_info.tx.hash.to_string(),
                    "Device submitted to blockchain"
                );
                device.web3 = Some(web3_info);
                match self.storage.update_device(device) {
                    Ok(_) => Ok(()),
                    Err(storage_error) => return Err(BitacoraError::StorageError(storage_error)),
                }
            }
            Err(_) => Err(BitacoraError::Web3Error),
        }
    }

    async fn timestamp_dataset(
        &self,
        dataset: &mut Dataset,
        device_id: &String,
        flight_datas: &[FlightData],
    ) -> Result<(), BitacoraError> {
        match self
            .timestamper
            .register_dataset(dataset, device_id, flight_datas)
            .await
        {
            Ok(mut web3_info) => {
                info!(
                    dataset = dataset.id,
                    tx_hash = web3_info.tx.hash.to_string(),
                    "Dataset submitted to blockchain"
                );
                web3_info.merkle_receipt = match web3_info.merkle_receipt.unwrap() {
                    MerkleTreeOZReceipt::Tree(ref mut mt) => {
                        Some(MerkleTreeOZReceipt::Root(mt.root().unwrap()))
                    }
                    _ => unreachable!(),
                };
                dataset.web3 = Some(web3_info);
                Ok(())
            }
            Err(_err) => Err(BitacoraError::Web3Error),
        }
    }

    pub fn get_flight_data_receipt(&self, fd: &FlightData) -> Result<Web3Info, BitacoraError> {
        let dataset = self.storage.get_flight_data_dataset(&fd.id)?;
        let fds = self.storage.get_dataset_flight_datas(&dataset.id)?;
        let dataset_receipt = match dataset.web3 {
            Some(receipt) => receipt,
            None => return Err(BitacoraError::Web3Error),
        };
        Ok(T::flight_data_web3_info(fd, &fds, &dataset_receipt)?)
    }
}

impl<S: FullStorage, T: Timestamper> DeviceStorage for SharedBitacora<S, T> {
    fn new_device(&self, device: &Device, dataset_limit: u32) -> Result<(), StorageError> {
        self.storage.new_device(device, dataset_limit)
    }

    fn get_device(&self, id: &DeviceId) -> Result<Device, StorageError> {
        self.storage.get_device(id)
    }

    fn update_device(&self, device: &Device) -> Result<(), StorageError> {
        self.storage.update_device(device)
    }
}

impl<S: FullStorage, T: Timestamper> FlightDataStorage for SharedBitacora<S, T> {
    fn get_flight_data(&self, id: &FlightDataId) -> Result<FlightData, StorageError> {
        self.storage.get_flight_data(id)
    }

    fn new_flight_data(
        &self,
        fd: &FlightData,
        device_id: &DeviceId,
    ) -> Result<Dataset, StorageError> {
        self.storage.new_flight_data(fd, device_id)
    }

    fn get_dataset_flight_datas(&self, ds_id: &DatasetId) -> Result<Vec<FlightData>, StorageError> {
        self.storage.get_dataset_flight_datas(ds_id)
    }

    fn get_flight_data_dataset(&self, fd_id: &FlightDataId) -> Result<Dataset, StorageError> {
        self.storage.get_flight_data_dataset(fd_id)
    }

    fn new_dataset(&self, limit: u32, device_id: &DeviceId) -> Result<Dataset, StorageError> {
        self.storage.new_dataset(limit, device_id)
    }

    fn get_dataset(&self, id: &DatasetId) -> Result<Dataset, StorageError> {
        self.storage.get_dataset(id)
    }

    fn get_latest_dataset(&self, device_id: &DeviceId) -> Result<Option<Dataset>, StorageError> {
        self.storage.get_latest_dataset(device_id)
    }

    fn update_dataset_web3(&self, ds: &Dataset) -> Result<(), StorageError> {
        self.storage.update_dataset_web3(ds)
    }
}
