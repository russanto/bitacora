use std::sync::Arc;

use tracing::{error, warn, info, debug, trace};

use crate::storage::storage::{FullStorage, FlightDataStorage, DeviceStorage};

use super::entities::{FlightData, DeviceId, Dataset, FlightDataId};
use super::errors::{NewFlightDataError, NewDatasetError};

const DATASET_DEFAULT_LIMIT: u32 = 10;  //TODO: refactor with configuration management

type SharedBitacora<S> = Arc<Bitacora<S>>;

pub struct Bitacora<S: FullStorage> {
    pub storage: S
}

impl <S: FullStorage> Bitacora<S> {
    pub fn new_flight_data(&self, fd: &FlightData, device_id: &DeviceId) -> Result<Dataset, NewFlightDataError> {
        info!("Creating a new FlightData");
        trace!(device_id = device_id, "Searching the supplied device");
        let _ = match self.storage.get_device(device_id) {
            Ok(maybe_device) => match maybe_device {
                Some(device) => device,
                None => return Err(NewFlightDataError::DeviceNotFound)
            },
            Err(_) => return Err(NewFlightDataError::StorageError)
        };
        trace!(flight_data_id = fd.id, "Storing the FlightData");
        match self.storage.set_flight_data(fd) {
            Ok(already_existing) => {
                if already_existing {
                    warn!(flight_data_id=fd.id, "Supplied FlightData already exists");
                    return Err(NewFlightDataError::AlreadyExists)
                }
            }
            Err(_) => return Err(NewFlightDataError::StorageError)
        };
        trace!(device_id = device_id, "Getting the latest dataset");
        let dataset = match self.storage.get_latest_dataset(device_id) {
            Ok(maybe_dataset) => match maybe_dataset {
                Some(dataset) => {
                    if dataset.limit == dataset.count {
                        debug!(device_id = device_id, dataset_id = dataset.id, "Latest dataset is full");
                        Option::None
                    } else {
                        Option::Some(dataset)
                    }
                },
                None => {
                    debug!(device_id = device_id, "Device has no assigned dataset");
                    Option::None
                }
            },
            Err(_) => return Err(NewFlightDataError::StorageError)
        };
        let dataset = match dataset {
            Some(ds) => ds,
            None => match self.new_dataset(DATASET_DEFAULT_LIMIT, device_id) {
                Ok(new_dataset) => new_dataset,
                //TODO handle failure here (FlightData has no Dataset)
                Err(_) => return Err(NewFlightDataError::StorageError)
            }
        };
        trace!(dataset_id = dataset.id, flight_data_id = fd.id, "Adding FlightData to the Dataset");
        match self.storage.add_flight_data(&dataset.id, fd) {
            Ok(_) => {
                trace!(flight_data_id=fd.id, "Created FlightData");
                Ok(dataset)
            },
            Err(_) => Err(NewFlightDataError::StorageError)
        }
    }

    pub fn new_dataset(&self, limit: u32, device_id: &DeviceId) -> Result<Dataset, NewDatasetError> {
        trace!(device_id=device_id, "Creating new Dataset");
        let new_id = match self.storage.new_dataset_id() {
            Ok(id) => id,
            Err(_) => return Err(NewDatasetError::StorageError)
        };
        let dataset = Dataset {
            id: new_id.clone(),
            limit,
            count: 0,
            merkle_tree: Option::None
        };
        match self.storage.set_dataset(&dataset, device_id) {
            Ok(_) => { //TODO: manage clashes on Ids
                trace!(dataset_id=new_id, device_id=device_id, "Created Dataset");
                Ok(dataset)
            }, 
            Err(_) => Err(NewDatasetError::StorageError)
        }
    }
}

impl <S: FullStorage> FlightDataStorage for SharedBitacora<S> {
    fn get_flight_data(&self, id: &FlightDataId) -> Result<Option<FlightData>, crate::storage::errors::Error> {
        self.storage.get_flight_data(id)
    }

    fn set_flight_data(&self, fd: &FlightData) -> Result<bool, crate::storage::errors::Error> {
        self.storage.set_flight_data(fd)
    }
}

impl <S: FullStorage> DeviceStorage for SharedBitacora<S> {
    fn get_device(&self, id: &DeviceId) -> Result<Option<super::entities::Device>, crate::storage::errors::Error> {
        self.storage.get_device(id)
    }

    fn set_device(&self, device: &super::entities::Device) -> Result<bool, crate::storage::errors::Error> {
        self.storage.set_device(device)
    }
}