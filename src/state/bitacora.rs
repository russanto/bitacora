use std::sync::Arc;

use tracing::{warn, info, debug, trace};

use crate::storage::errors::Error as StorageError;
use crate::storage::storage::{FullStorage, FlightDataStorage, DeviceStorage, DatasetStorage};
use crate::web3::traits::Timestamper;

use super::entities::{FlightData, Device, DeviceId, Dataset, Entity, FlightDataId};
use super::errors::BitacoraError;

const DATASET_DEFAULT_LIMIT: u32 = 10;  //TODO: refactor with configuration management

type SharedBitacora<S, T> = Arc<Bitacora<S, T>>;

pub struct Bitacora<S, T> 
where
    S: FullStorage,
    T: Timestamper
{
    storage: S,
    timestamper: T
}

impl <S, T> Bitacora<S, T>
where
    S: FullStorage,
    T: Timestamper
{
    pub fn new(storage: S, timestamper: T) -> Bitacora<S, T> {
        Bitacora { storage, timestamper }
    }

    pub async fn new_flight_data(&self, fd: &FlightData, device_id: &DeviceId) -> Result<Dataset, BitacoraError> {
        info!("Creating a new FlightData");
        trace!(device_id = device_id, "Searching the supplied device");
        let _ = match self.storage.get_device(device_id) {
            Ok(maybe_device) => match maybe_device {
                Some(device) => device,
                None => return Err(BitacoraError::NotFound)
            },
            Err(storage_error) => return Err(BitacoraError::StorageError(storage_error))
        };
        trace!(flight_data_id = fd.id.to_string(), "Storing the FlightData");
        match self.storage.set_flight_data(fd) {
            Ok(already_existing) => {
                if already_existing {
                    warn!(flight_data_id=fd.id.to_string(), "Supplied FlightData already exists");
                    return Err(BitacoraError::AlreadyExists(Entity::FlightData, fd.id.clone().into()))
                }
            }
            Err(storage_error) => return Err(BitacoraError::StorageError(storage_error))
        };
        trace!(device_id = device_id, "Getting the latest dataset");
        let dataset = match self.storage.get_latest_dataset(device_id) {
            Ok(maybe_dataset) => match maybe_dataset {
                Some(dataset) => {
                    if dataset.limit == dataset.count {
                        debug!(device_id = device_id, dataset_id = dataset.id, "Latest dataset is full");
                        None
                    } else {
                        Some(dataset)
                    }
                },
                None => {
                    debug!(device_id = device_id, "Device has no assigned dataset");
                    None
                }
            },
            Err(storage_error) => return Err(BitacoraError::StorageError(storage_error))
        };
        let mut dataset = match dataset {
            Some(ds) => ds,
            None => match self.new_dataset(DATASET_DEFAULT_LIMIT, device_id) {
                Ok(new_dataset) => new_dataset,
                //TODO handle failure here (FlightData has no Dataset)
                Err(bitacora_error) => return Err(bitacora_error)
            }
        };
        trace!(dataset_id = dataset.id, flight_data_id = fd.id.to_string(), "Adding FlightData to the Dataset");
        match self.storage.add_flight_data(&dataset.id, fd) {
            Ok(_) => {
                trace!(flight_data_id=fd.id.to_string(), "Created FlightData");
                dataset.count += 1; // to avoid reading it again
            },
            Err(storage_error) => return Err(BitacoraError::StorageError(storage_error))
        }
        if dataset.count == dataset.limit {
            self.timestamp_dataset(&mut dataset, device_id).await?;
        }
        Ok(dataset)
    }

    pub fn new_dataset(&self, limit: u32, device_id: &DeviceId) -> Result<Dataset, BitacoraError> {
        trace!(device_id=device_id, "Creating new Dataset");
        let new_id = match self.storage.new_dataset_id() {
            Ok(id) => id,
            Err(storage_error) => return Err(BitacoraError::StorageError(storage_error))
        };
        let dataset = Dataset {
            id: new_id.clone(),
            limit,
            count: 0,
            merkle_tree: None,
            web3: None
        };
        match self.storage.add_dataset(&dataset, device_id) {
            Ok(_) => { //TODO: manage clashes on Ids
                trace!(dataset_id=new_id, device_id=device_id, "Created Dataset");
                Ok(dataset)
            }, 
            Err(storage_error) => match storage_error {
                StorageError::AlreadyExists => return Err(BitacoraError::AlreadyExists(Entity::Dataset, dataset.id)),
                _ => return Err(BitacoraError::StorageError(storage_error))
            }
        }
    }

    pub async fn new_device(&self, device: &mut Device) -> Result<(), BitacoraError> {
        match self.storage.new_device(&device) {
            Ok(_) => (),
            Err(storage_error) => match storage_error {
                StorageError::AlreadyExists => return Err(BitacoraError::AlreadyExists(Entity::Device, device.id.clone())),
                _ => return Err(BitacoraError::StorageError(storage_error))
            }
        };
        self.timestamp_device(device).await
    }

    async fn timestamp_device(&self, device: &mut Device) -> Result<(), BitacoraError> {
        match self.timestamper.register_device(device).await {
            Ok(web3_info) => {
                info!(device=device.id, tx_hash=web3_info.tx.hash.to_string(), "Device submitted to blockchain");
                device.web3 = Some(web3_info);
                match self.storage.set_device(device) {
                    Ok(_) => Ok(()),
                    Err(storage_error) => return Err(BitacoraError::StorageError(storage_error))
                }
            },
            Err(_) => Err(BitacoraError::Web3Error)
        }
    }

    async fn timestamp_dataset(&self, dataset: &mut Dataset, device_id: &String) -> Result<(), BitacoraError> {
        match self.timestamper.register_dataset(dataset, device_id).await {
            Ok(web3_info) => {
                info!(dataset=dataset.id, tx_hash=web3_info.tx.hash.to_string(), "Dataset submitted to blockchain");
                dataset.web3 = Some(web3_info);
                match self.storage.set_dataset(dataset) {
                    Ok(_) => Ok(()),
                    Err(storage_error) => return Err(BitacoraError::StorageError(storage_error))
                }
            },
            Err(_) => Err(BitacoraError::Web3Error)
        }
    }
}

impl <S: FullStorage, T: Timestamper> FlightDataStorage for SharedBitacora<S, T> {
    fn get_flight_data(&self, id: &FlightDataId) -> Result<Option<FlightData>, crate::storage::errors::Error> {
        self.storage.get_flight_data(id)
    }

    fn set_flight_data(&self, fd: &FlightData) -> Result<bool, crate::storage::errors::Error> {
        self.storage.set_flight_data(fd)
    }
}

impl <S: FullStorage, T: Timestamper> DeviceStorage for SharedBitacora<S, T> {
    fn new_device(&self, device: &Device) -> Result<(), StorageError> {
        self.storage.new_device(device)
    }

    fn get_device(&self, id: &DeviceId) -> Result<Option<super::entities::Device>, crate::storage::errors::Error> {
        self.storage.get_device(id)
    }

    fn set_device(&self, device: &super::entities::Device) -> Result<bool, crate::storage::errors::Error> {
        self.storage.set_device(device)
    }
}

impl <S: FullStorage, T: Timestamper> DatasetStorage for SharedBitacora<S, T> {
    fn add_flight_data(&self, ds_id: &super::entities::DatasetId, fd: &FlightData) -> Result<(), crate::storage::errors::Error> {
        self.storage.add_flight_data(ds_id, fd)
    }

    fn add_dataset(&self, ds: &Dataset, device_id: &DeviceId) -> Result<(), crate::storage::errors::Error> {
        self.storage.add_dataset(ds, device_id)
    }

    fn set_dataset(&self, ds: &Dataset) -> Result<bool, crate::storage::errors::Error> {
        self.storage.set_dataset(ds)
    }

    fn get_dataset(&self, id: &super::entities::DatasetId) -> Result<Option<Dataset>, crate::storage::errors::Error> {
        self.storage.get_dataset(id)
    }

    fn get_latest_dataset(&self, device_id: &DeviceId) -> Result<Option<Dataset>, crate::storage::errors::Error> {
        self.storage.get_latest_dataset(device_id)
    }

    fn new_dataset_id(&self) -> Result<super::entities::DatasetId, crate::storage::errors::Error> {
        self.storage.new_dataset_id()
    }
}