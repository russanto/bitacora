use std::sync::Arc;

use tracing::{warn, info, debug, trace};

use crate::storage::storage::{FullStorage, FlightDataStorage, DeviceStorage, DatasetStorage};
use crate::web3::traits::Timestamper;

use super::entities::{FlightData, DeviceId, Dataset, FlightDataId};
use super::errors::BitacoraError;

const DATASET_DEFAULT_LIMIT: u32 = 2;  //TODO: refactor with configuration management

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

    pub fn new_flight_data(&self, fd: &FlightData, device_id: &DeviceId) -> Result<Dataset, BitacoraError> {
        info!("Creating a new FlightData");
        trace!(device_id = device_id, "Searching the supplied device");
        let _ = match self.storage.get_device(device_id) {
            Ok(maybe_device) => match maybe_device {
                Some(device) => device,
                None => return Err(BitacoraError::NotFound)
            },
            Err(_) => return Err(BitacoraError::StorageError)
        };
        trace!(flight_data_id = fd.id, "Storing the FlightData");
        match self.storage.set_flight_data(fd) {
            Ok(already_existing) => {
                if already_existing {
                    warn!(flight_data_id=fd.id, "Supplied FlightData already exists");
                    return Err(BitacoraError::AlreadyExists)
                }
            }
            Err(_) => return Err(BitacoraError::StorageError)
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
            Err(_) => return Err(BitacoraError::StorageError)
        };
        let mut dataset = match dataset {
            Some(ds) => ds,
            None => match self.new_dataset(DATASET_DEFAULT_LIMIT, device_id) {
                Ok(new_dataset) => new_dataset,
                //TODO handle failure here (FlightData has no Dataset)
                Err(_) => return Err(BitacoraError::StorageError)
            }
        };
        trace!(dataset_id = dataset.id, flight_data_id = fd.id, "Adding FlightData to the Dataset");
        match self.storage.add_flight_data(&dataset.id, fd) {
            Ok(_) => {
                trace!(flight_data_id=fd.id, "Created FlightData");
                dataset.count += 1; // to avoid reading it again
            },
            Err(_) => return Err(BitacoraError::StorageError)
        }
        if dataset.count == dataset.limit {
            self.timestamp_dataset(&mut dataset)?;
        }
        Ok(dataset)
    }

    pub fn new_dataset(&self, limit: u32, device_id: &DeviceId) -> Result<Dataset, BitacoraError> {
        trace!(device_id=device_id, "Creating new Dataset");
        let new_id = match self.storage.new_dataset_id() {
            Ok(id) => id,
            Err(_) => return Err(BitacoraError::StorageError)
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
            Err(_) => Err(BitacoraError::StorageError)
        }
    }

    fn timestamp_dataset(&self, dataset: &mut Dataset) -> Result<(), BitacoraError> {
        match self.timestamper.register_dataset(dataset) {
            Ok(web3_info) => {
                info!(dataset=dataset.id, tx_hash=web3_info.tx.hash, "Dataset submitted to blockchain");
                dataset.web3 = Some(web3_info);
                match self.storage.set_dataset(dataset) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(BitacoraError::StorageError)
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