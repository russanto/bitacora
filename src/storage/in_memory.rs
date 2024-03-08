use std::collections::HashMap;
use std::sync::RwLock;

use sha2::{Digest, Sha256};

use crate::configuration::BitacoraConfiguration;
use crate::state::entities::{
    Dataset, DatasetId, Device, DeviceId, Entity, FlightData, FlightDataId,
};

use super::errors::Error;
use super::storage::{DeviceStorage, FlightDataStorage, FullStorage};

#[derive(Default)]
pub struct InMemoryStorage {
    devices: RwLock<HashMap<DeviceId, Device>>,
    fligth_data: RwLock<HashMap<FlightDataId, FlightData>>,
    datasets: RwLock<HashMap<DatasetId, Dataset>>,
    datasets_flight_datas: RwLock<HashMap<DatasetId, Vec<FlightDataId>>>,
    flight_data_dataset: RwLock<HashMap<FlightDataId, DatasetId>>,
    devices_datasets: RwLock<HashMap<DeviceId, Vec<DatasetId>>>,
    dataset_limits: RwLock<HashMap<DeviceId, u32>>,
}

impl InMemoryStorage {
    fn new_dataset_id() -> DatasetId {
        let mut hasher = Sha256::new();
        hasher.update(rand::random::<u64>().to_be_bytes());
        hasher.update(rand::random::<u64>().to_be_bytes());
        bs58::encode(hasher.finalize()).into_string()
    }

    fn increment_dataset_count(&self, ds_id: &DatasetId) -> Result<u32, Error> {
        let mut datasets_write_access = self.datasets.write().unwrap();
        let dataset = match datasets_write_access.get_mut(ds_id) {
            Some(ds) => ds,
            None => return Err(Error::NotFound(Entity::Dataset)),
        };
        if dataset.count >= dataset.limit {
            return Err(Error::Generic);
        }
        dataset.count += 1;
        Ok(dataset.count)
    }
}

impl DeviceStorage for InMemoryStorage {
    fn new_device(&self, device: &Device, dataset_limit: u32) -> Result<(), Error> {
        let mut write_lock = self.devices.write().unwrap();
        if write_lock.contains_key(&device.id) {
            return Err(Error::AlreadyExists);
        }
        write_lock.insert(device.id.clone(), device.clone());
        self.devices_datasets
            .write()
            .unwrap()
            .insert(device.id.clone(), vec![]);
        self.dataset_limits
            .write()
            .unwrap()
            .insert(device.id.clone(), dataset_limit);
        Ok(())
    }

    fn update_device(&self, device: &Device) -> Result<(), Error> {
        match self
            .devices
            .write()
            .unwrap()
            .insert(device.id.clone(), device.clone())
        {
            Some(_) => Ok(()),
            None => Err(Error::NotFound(Entity::Device)),
        }
    }

    fn get_device(&self, id: &DeviceId) -> Result<Device, Error> {
        let acquire_read = self.devices.read().unwrap();
        match acquire_read.get(id) {
            Some(device) => Ok(device.to_owned()),
            None => Err(Error::NotFound(Entity::Device)),
        }
    }
}

impl FlightDataStorage for InMemoryStorage {
    fn new_flight_data(&self, fd: &FlightData, device_id: &DeviceId) -> Result<Dataset, Error> {
        // This ensure exclusive access on the new flight data process
        let mut fd_write_access = self.fligth_data.write().unwrap();
        if let Some(already_fd) = fd_write_access.insert(fd.id.clone(), fd.clone()) {
            fd_write_access.insert(fd.id.clone(), already_fd);
            return Err(Error::AlreadyExists);
        }
        let mut dataset: Option<Dataset> = Option::None;
        {
            let devices_datasets_read_lock = self.devices_datasets.read().unwrap();
            match devices_datasets_read_lock.get(device_id) {
                Some(dataset_list) => match dataset_list.last() {
                    Some(ds_id) => {
                        let dataset_candidate = self.get_dataset(ds_id).unwrap();
                        if dataset_candidate.count < dataset_candidate.limit {
                            dataset = Some(dataset_candidate);
                        }
                    }
                    None => (),
                },
                None => return Err(Error::NotFound(Entity::Device)),
            };
        }
        if dataset.is_none() {
            let dataset_limit = self
                .dataset_limits
                .read()
                .unwrap()
                .get(device_id)
                .unwrap()
                .clone();
            dataset = Some(self.new_dataset(dataset_limit, device_id).unwrap());
        }
        let mut dataset = dataset.unwrap();
        //next line should never fail thanks to the fd_write_access lock
        dataset.count = self.increment_dataset_count(&dataset.id).unwrap();
        self.datasets_flight_datas
            .write()
            .unwrap()
            .entry(dataset.id.clone())
            .and_modify(|fd_list| fd_list.push(fd.id.clone()));
        self.flight_data_dataset
            .write()
            .unwrap()
            .insert(fd.id.clone(), dataset.id.clone());
        Ok(dataset)
    }

    fn get_flight_data(&self, id: &FlightDataId) -> Result<FlightData, Error> {
        match self.fligth_data.read().unwrap().get(id) {
            Some(fd) => Ok(fd.clone()),
            None => Err(Error::NotFound(Entity::FlightData)),
        }
    }

    fn get_dataset_flight_datas(&self, ds_id: &DatasetId) -> Result<Vec<FlightData>, Error> {
        let read_access_cross_ds_fd = self.datasets_flight_datas.read().unwrap();
        let read_access_fds = self.fligth_data.read().unwrap();
        let fd_ids = match read_access_cross_ds_fd.get(ds_id) {
            Some(fd_ids) => fd_ids,
            None => return Err(Error::NotFound(Entity::Dataset)),
        };
        let mut fds = Vec::new();
        for fd_id in fd_ids {
            fds.push(read_access_fds.get(fd_id).unwrap().clone());
        }
        Ok(fds)
    }

    fn get_flight_data_dataset(&self, fd_id: &FlightDataId) -> Result<Dataset, Error> {
        let ra_flight_data_dataset = self.flight_data_dataset.read().unwrap();
        let ds_id = ra_flight_data_dataset.get(fd_id);
        Ok(self
            .datasets
            .read()
            .unwrap()
            .get(ds_id.unwrap())
            .unwrap()
            .clone())
    }

    fn get_dataset(&self, id: &DatasetId) -> Result<Dataset, Error> {
        match self.datasets.read().unwrap().get(id) {
            Some(ds) => Ok(ds.clone()),
            None => Err(Error::NotFound(Entity::Dataset)),
        }
    }

    fn new_dataset(&self, limit: u32, device_id: &DeviceId) -> Result<Dataset, Error> {
        let dataset = Dataset {
            id: Self::new_dataset_id(),
            limit,
            count: 0,
            web3: None,
        };
        let mut datasets_write_lock = self.datasets.write().unwrap();
        let mut devices_datasets_write_lock = self.devices_datasets.write().unwrap();
        let mut datasets_flight_datas_write_lock = self.datasets_flight_datas.write().unwrap();
        match devices_datasets_write_lock.get_mut(device_id) {
            Some(dataset_list) => dataset_list.push(dataset.id.clone()),
            None => return Err(Error::NotFound(Entity::Device)),
        }
        datasets_write_lock.insert(dataset.id.clone(), dataset.clone());
        datasets_flight_datas_write_lock.insert(dataset.id.clone(), vec![]);
        Ok(dataset)
    }

    fn update_dataset_web3(&self, ds: &Dataset) -> Result<(), Error> {
        self.datasets
            .write()
            .unwrap()
            .entry(ds.id.clone())
            .and_modify(|dataset| {
                dataset.web3 = ds.web3.clone();
            });
        Ok(())
    }

    fn get_latest_dataset(&self, device_id: &DeviceId) -> Result<Option<Dataset>, Error> {
        match self.devices_datasets.read().unwrap().get(device_id) {
            Some(dataset_list) => {
                match dataset_list.last() {
                    Some(dataset_id) => match self.datasets.read().unwrap().get(dataset_id) {
                        Some(dataset) => Ok(Some(dataset.clone())),
                        None => panic!("Inconsistency on the InMemoryStorage. There is a dataset Id without its data")
                    },
                    None => Ok(Option::None)
                }
            },
            None => Err(Error::NotFound(Entity::Device))
        }
    }
}

impl FullStorage for InMemoryStorage {}
