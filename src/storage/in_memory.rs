use std::collections::HashMap;
use std::sync::RwLock;

use sha2::{Digest, Sha256};

use crate::state::entities::{Device, FlightData, Dataset, DeviceId, FlightDataId, DatasetId};

use super::errors::Error;
use super::storage::{FullStorage, DatasetStorage, DeviceStorage, FlightDataStorage};

#[derive(Default)]
pub struct InMemoryStorage {
    devices: RwLock<HashMap<DeviceId, Device>>,
    fligth_data: RwLock<HashMap<FlightDataId, FlightData>>,
    datasets: RwLock<HashMap<DatasetId, Dataset>>,
    datasets_flight_data: RwLock<HashMap<DatasetId, Vec<FlightDataId>>>,
    devices_datasets: RwLock<HashMap<DeviceId, Vec<DatasetId>>>
}

impl DeviceStorage for InMemoryStorage {
    fn new_device(&self, device: &Device) -> Result<(), Error> {
        let mut write_lock = self.devices.write().unwrap();
        if write_lock.contains_key(&device.id) {
            return Err(Error::AlreadyExists);
        }
        write_lock.insert(device.id.clone(), device.clone());
        Ok(())
    }

    fn set_device(&self, device: &Device) -> Result<bool, Error> {
        match self.devices.write().unwrap().insert(device.id.clone(), device.clone()) {
            Some(_) => Ok(true),
            None => {
                self.devices_datasets.write().unwrap().insert(device.id.clone(), vec![]);
                Ok(false)
            }
        }
    }

    fn get_device(&self, id: &DeviceId) -> Result<Option<Device>, Error> {
        let acquire_read = self.devices.read().unwrap();
        let result = acquire_read.get(id);
        if result.is_some() {
            Ok(Some(result.unwrap().clone()))
        } else {
            Ok(Option::None)
        }
    }
}

impl FlightDataStorage for InMemoryStorage {
    fn set_flight_data(&self, fd: &FlightData) -> Result<bool, Error> {
        match self.fligth_data.write().unwrap().insert(fd.id.clone(), fd.clone()) {
            Some(_) => Ok(true),
            None => Ok(false)
        }   
    }

    fn get_flight_data(&self, id: &FlightDataId) -> Result<Option<FlightData>, Error> {
        let acquire_read = self.fligth_data.read().unwrap();
        let result = acquire_read.get(id);
        if result.is_some() {
            Ok(Some(result.unwrap().clone()))
        } else {
            Ok(Option::None)
        }   
    }
}

impl DatasetStorage for InMemoryStorage {
    fn add_flight_data(&self, ds_id: &DatasetId, fd: &FlightData) -> Result<(), Error> {
        let mut write_access = self.datasets_flight_data.write().unwrap();
        match write_access.get_mut(ds_id) {
            Some(vector) => {
                vector.push(fd.id.clone());
            },
            None => return Err(Error::InconsistentRelatedData(String::from("Dataset"), String::from("FlightData")))
        };
        let mut write_access = self.datasets.write().unwrap();
        match write_access.get_mut(ds_id) {
            Some(dataset) => dataset.count += 1,
            None => unreachable!()
        }
        Ok(())
    }

    fn get_dataset(&self, id: &DatasetId) -> Result<Option<Dataset>, Error> {
        match self.datasets.read().unwrap().get(id) {
            Some(dataset) => Ok(Some(dataset.clone())),
            None => Ok(Option::None)
        }
    }

    fn set_dataset(&self, ds: &Dataset) -> Result<bool, Error> {
        match self.datasets.write().unwrap().insert(ds.id.clone(), ds.clone()) {
            Some(_) => Ok(true),
            None => Ok(false)
        }
    }

    fn add_dataset(&self, ds: &Dataset, device_id: &DeviceId) -> Result<(), Error> {
        let mut write_lock = self.datasets.write().unwrap();
        match write_lock.insert(ds.id.clone(), ds.clone()) {
            Some(old_dataset) => { //a dataset already exists with such ID this is an error
                write_lock.insert(old_dataset.id.clone(), old_dataset); // restore the old dataset
                Err(Error::AlreadyExists)
            },
            None => {
                let mut write_access = self.devices_datasets.write().unwrap();
                match write_access.get_mut(device_id) {
                    Some(dataset_list) => dataset_list.push(ds.id.clone()),
                    None => return Err(Error::FailedRelatingData(String::from("Dataset"), String::from("Device")))
                }
                self.datasets_flight_data.write().unwrap().insert(ds.id.clone(), vec![]);
                Ok(())
            }
        }
    }

    fn get_latest_dataset(&self, device_id: &DeviceId) -> Result<Option<Dataset>, Error> {
        match self.devices_datasets.read().unwrap().get(device_id) {
            Some(dataset_list) => {
                match dataset_list.last() {
                    Some(dataset_id) => match self.datasets.read().unwrap().get(dataset_id) {
                        Some(dataset) => Ok(Some(dataset.clone())),
                        None => Err(Error::NotFound(String::from("Latest Dataset Id does not have corresponding data")))
                    },
                    None => Ok(Option::None)
                }
                
            },
            None => Err(Error::NotFound(String::from("Device not found")))
        }
    }

    fn new_dataset_id(&self) -> Result<DatasetId, Error> {
        let mut hasher = Sha256::new();
        hasher.update(rand::random::<u64>().to_be_bytes());
        hasher.update(rand::random::<u64>().to_be_bytes());
        Ok(bs58::encode(hasher.finalize()).into_string())
    }
}

impl FullStorage for InMemoryStorage {}