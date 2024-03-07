// use std::sync::{Arc, RwLock};

use crate::state::entities::{Device, FlightData, Dataset, DatasetId, DeviceId, FlightDataId};

use super::errors::Error;

/// Trait defining storage operations for devices.
pub trait DeviceStorage {
    /// Creates a new device. The Id of the Device is provided as input and an error is returned if it already exists.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the `Device` to be added.
    ///
    /// # Returns
    ///
    /// `Result<(), Error>` - Ok if the device was successfully created, Err otherwise.
    fn new_device(&self, device: &Device, dataset_limit: u32) -> Result<(), Error>;

    /// Updates an existing device.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the `Device` to be updated.
    ///
    /// # Returns
    ///
    /// `Result<bool, Error>` - Ok(()) if the device was successfully updated, Err if an error occurred.
    fn update_device(&self, device: &Device) -> Result<(), Error>;

    /// Retrieves a device by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the device to retrieve.
    ///
    /// # Returns
    ///
    /// `Result<Device, Error>` - Ok(device) if found, Err if an error occurred.
    fn get_device(&self, id: &DeviceId) -> Result<Device, Error>;
}


/// Trait defining storage operations for flight data and datasets.
pub trait FlightDataStorage {
    /// Adds new flight data for a specific device and returns the dataset ID it was added to.
    ///
    /// This function takes flight data and a device ID, adding the flight data to the appropriate dataset
    /// associated with the device. It might create a new dataset if necessary or add to an existing one, 
    /// depending on the implementation.
    ///
    /// # Arguments
    ///
    /// * `fd` - Reference to the flight data to be added.
    /// * `device_id` - ID of the device associated with the flight data.
    ///
    /// # Returns
    ///
    /// `Result<DatasetId, Error>` - Ok(dataset_id) if the flight data was successfully added to a dataset,
    /// Err if an error occurred during the operation.
    fn new_flight_data(&self, fd: &FlightData, device_id: &DeviceId) -> Result<Dataset, Error>;

    /// Retrieves flight data by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the flight data to retrieve.
    ///
    /// # Returns
    ///
    /// `Result<FlightData, Error>` - Ok(flight_data) if found, Err if an error occurred.
    fn get_flight_data(&self, id: &FlightDataId) -> Result<FlightData, Error>;

    /// Creates a new dataset associated with a device, which becomes the target for all new
    /// FlightDatas
    ///
    /// # Arguments
    ///
    /// * `ds` - Reference to the dataset to be created.
    /// * `device_id` - ID of the device associated with the dataset.
    ///
    /// # Returns
    ///
    /// `Result<(), Error>` - Ok if the dataset was successfully created, Err otherwise.
    fn new_dataset(&self, limit: u32, device_id: &DeviceId) -> Result<Dataset, Error>;

    /// Retrieves a dataset by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the dataset to retrieve.
    ///
    /// # Returns
    ///
    /// `Result<Dataset, Error>` - Ok(dataset) if found, Err if an error occurred.
    fn get_dataset(&self, id: &DatasetId) -> Result<Dataset, Error>;

    /// Updates an existing dataset in the storage.
    ///
    /// This method updates the details of an existing dataset based on the provided `Dataset` instance.
    /// It is expected that this method will only succeed if a dataset with the same ID as the provided
    /// `ds` already exists in the storage. If no matching dataset is found, the operation fails.
    ///
    /// # Arguments
    ///
    /// * `ds` - A reference to the `Dataset` containing the updated information.
    ///
    /// # Returns
    ///
    /// `Result<(), Error>` - Ok(()) if the dataset was successfully updated in the storage,
    /// Err if any error occurred during the operation.
    fn update_dataset_web3(&self, ds: &Dataset) -> Result<(), Error>;

    /// Retrieves the latest dataset for a given device.
    ///
    /// # Arguments
    ///
    /// * `device_id` - The ID of the device to retrieve the latest dataset for.
    ///
    /// # Returns
    ///
    /// `Result<Option<Dataset>, Error>` - Ok(Some(dataset)) if found, Ok(None) if there are no datasets for the device, Err if an error occurred.
    fn get_latest_dataset(&self, device_id: &DeviceId) -> Result<Option<Dataset>, Error>;

    /// Retrieves all flight data entries for a specific dataset.
    ///
    /// # Arguments
    ///
    /// * `ds_id` - The ID of the dataset to retrieve flight data for.
    ///
    /// # Returns
    ///
    /// `Result<Vec<FlightData>, Error>` - A vector of flight data entries, or Err if an error occurred.
    fn get_dataset_flight_datas(&self, ds_id: &DatasetId) -> Result<Vec<FlightData>, Error>;

    /// Retrieves the dataset associated with a specific flight data entry.
    ///
    /// # Arguments
    ///
    /// * `fd_id` - The ID of the flight data to find the associated dataset for.
    ///
    /// # Returns
    ///
    /// `Result<Dataset, Error>` - The associated dataset if found, or Err if an error occurred.
    fn get_flight_data_dataset(&self, fd_id: &FlightDataId) -> Result<Dataset, Error>;
}


pub trait FullStorage: DeviceStorage + FlightDataStorage {}

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