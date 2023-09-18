// use std::sync::{Arc, RwLock};

use crate::state::entities::{Device, FlightData, Dataset, DatasetId, DeviceId, FlightDataId};

use super::errors::Error;

pub trait DeviceStorage {
    fn new_device(&self, device: &Device) -> Result<(), Error>; 
    fn set_device(&self, device: &Device) -> Result<bool, Error>; 
    fn get_device(&self, id: &DeviceId) -> Result<Option<Device>, Error>;
}

pub trait FlightDataStorage {
    fn set_flight_data(&self, fd: &FlightData) -> Result<bool, Error>;
    fn get_flight_data(&self, id: &FlightDataId) -> Result<Option<FlightData>, Error>;
}

pub trait DatasetStorage {
    fn set_dataset(&self, ds: &Dataset) -> Result<bool, Error>;
    fn add_dataset(&self, ds: &Dataset, device_id: &DeviceId) -> Result<(), Error>;
    fn get_dataset(&self, id: &DatasetId) -> Result<Option<Dataset>, Error>;
    fn get_latest_dataset(&self, device_id: &DeviceId) -> Result<Option<Dataset>, Error>;
    fn add_flight_data(&self, ds_id: &DatasetId, fd: &FlightData) -> Result<(), Error>;
    fn new_dataset_id(&self) -> Result<DatasetId, Error>;
}

pub trait FullStorage: DatasetStorage + DeviceStorage + FlightDataStorage {}

// pub type ThreadSafeStorageWrapper<S> = Arc<RwLock<S>>;

// impl <S: FlightDataStorage> FlightDataStorage for ThreadSafeStorageWrapper<S> {
//     fn set_flight_data(&self, fd: &FlightData) -> Result<bool, Error> {
//         self.write().unwrap().set_flight_data(fd)
//     }

//     fn get_flight_data(&self, id: &str) -> Result<Option<FlightData>, Error> {
//         self.read().unwrap().get_flight_data(id)
//     }
// }

// impl <S: DeviceStorage> DeviceStorage for ThreadSafeStorageWrapper<S> {
//     fn get_device(&self, id: &str) -> Result<Option<Device>, Error> {
//         self.read().unwrap().get_device(id)
//     }

//     fn set_device(&self, device: &Device) -> Result<bool, Error> {
//         self.write().unwrap().set_device(device)
//     }
// }