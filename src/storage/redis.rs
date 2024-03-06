use std::collections::HashMap;
use std::sync::{ Arc, Mutex, MutexGuard };

use base64::{Engine as _, engine::general_purpose::STANDARD};

use redis::{Commands, RedisError, RedisResult, ToRedisArgs};
use serde::{Deserialize, Serialize};
use serde_json;

use sha2::{Digest, Sha256};
use tracing_subscriber::registry::Data;

use crate::common::prelude::Bytes32;
use crate::configuration::BitacoraConfiguration as Conf;
use crate::state::entities::{Dataset, DatasetCounter, DatasetId, Device, DeviceId, Entity, FlightData, FlightDataId, Timestamp};

use super::storage::{DeviceStorage, FlightDataStorage};
use super::errors::Error;

type Result<T> = std::result::Result<T, Error>;

type RedisKey = String;

const PREFIX_DEVICE_KEY: &str = "device";

struct RedisStorage {
    conn: Arc<Mutex<redis::Connection>>,
}

impl RedisStorage {
    pub fn new(connection_str: &str) -> Result<RedisStorage> {
        let client = redis::Client::open(connection_str)?;
        let conn = Arc::new(Mutex::new(client.get_connection()?));
        Ok(RedisStorage { conn })
    }

    fn get_device_key(id: &DeviceId) -> String {
        format!("{}:{}", PREFIX_DEVICE_KEY, id)
    }

    fn get_flight_data_key(id: &FlightDataId) -> String {
        format!("{}:{}", "flight_data", id)
    }

    fn get_device_flight_data_key(id: &DeviceId) -> String {
        format!("{}:{}", "device_flight_data", id)
    }

    fn get_dataset_key(id: &DatasetId) -> String {
        format!("{}:{}", "dataset", id)
    }

    fn get_dataset_flight_data_key(id: &DatasetId) -> String {
        format!("{}:{}", "dataset_flight_data", id)
    }

    fn get_device_fields(device: &Device) -> Vec<(&str, String)> {
        let mut fields = vec![
            ("id", device.id.to_string()),
            ("public_key", device.pk.to_string()),
            ("flight_data_count", "0".to_string()),
            ("dataset_limit", "10".to_string())
        ];
        if device.web3.is_some() {
            fields.push(("web3", serde_json::to_string(&device.web3).unwrap())) //TODO: replace with something more efficient
        }
        fields
    }

    fn get_flight_data_fields(fd: &FlightData, ds_id: &DatasetId) -> Vec<(&'static str, String)> {
        vec![
            ("id", fd.id.to_string()),
            ("signature", fd.signature.to_string()),
            ("timestamp", fd.timestamp.to_string()),
            ("localization", serde_json::to_string(&fd.localization).unwrap()),
            ("payload", STANDARD.encode(fd.payload.as_slice())),
            ("dataset", Self::get_dataset_key(ds_id))
        ]
    }

    fn get_dataset_fields(ds: &Dataset, device_id: &DeviceId) -> Vec<(&'static str, String)> {
        let mut fields = vec![
            ("id", ds.id.clone()),
            ("device", Self::get_device_key(device_id)),
            ("limit", ds.limit.to_string())
        ];
        if ds.web3.is_some() {
            fields.push(("web3", serde_json::to_string(&ds.web3).unwrap())) //TODO: replace with something more efficient
        }
        fields
    }

    fn no_lock_create_dataset(conn: &mut MutexGuard<redis::Connection>, dataset: &Dataset, device_id: &DeviceId) -> Result<()> {
        let dataset_key = Self::get_dataset_key(&dataset.id);
        let device_key = Self::get_device_key(device_id);
        conn.hset_multiple(dataset_key.clone(), &Self::get_dataset_fields(&dataset, device_id))?;
        conn.hset(device_key, "current_dataset", dataset_key)?;
        Ok(())
    }

    fn no_lock_get_flight_data(conn: &mut MutexGuard<redis::Connection>, key: RedisKey) -> Result<FlightData> {
        let flight_data_data: HashMap<String, String> = conn.hgetall(key)?;
        if flight_data_data.is_empty() {
            return Err(Error::NotFound(Entity::FlightData));
        }
        if !flight_data_data.contains_key("id") {
            return Err(Error::MalformedData("id".to_string()));
        }
        let id: FlightDataId = match flight_data_data.get("id").unwrap().try_into() {
            Ok(id) => id,
            Err(_) => return Err(Error::MalformedData("id".to_string()))
        };
        if !flight_data_data.contains_key("signature") {
            return Err(Error::MalformedData("signature".to_string()));
        }
        if !flight_data_data.contains_key("timestamp") {
            return Err(Error::MalformedData("timestamp".to_string()));
        }
        if !flight_data_data.contains_key("localization") {
            return Err(Error::MalformedData("localization".to_string()));
        }
        if !flight_data_data.contains_key("payload") {
            return Err(Error::MalformedData("payload".to_string()));
        }
        if !flight_data_data.contains_key("dataset") {
            return Err(Error::MalformedData("dataset".to_string()));
        }
        Ok(FlightData {
            id,
            signature: flight_data_data.get("signature").unwrap().clone(),
            timestamp: flight_data_data.get("timestamp").unwrap().parse().unwrap(),
            localization: serde_json::from_str(flight_data_data.get("localization").unwrap()).unwrap(),
            payload: STANDARD.decode(flight_data_data.get("payload").unwrap().as_bytes()).unwrap()
        })
    }

    fn no_lock_get_dataset(conn: &mut MutexGuard<redis::Connection>, key: RedisKey) -> Result<Dataset> {
        let dataset_data: HashMap<String, String> = conn.hgetall(key)?;
        if dataset_data.is_empty() {
            return Err(Error::NotFound(Entity::Dataset));
        }
        if !dataset_data.contains_key("id") {
            return Err(Error::MalformedData("id".to_string()));
        }
        if !dataset_data.contains_key("device") {
            return Err(Error::MalformedData("device".to_string()));
        }
        if !dataset_data.contains_key("limit") {
            return Err(Error::MalformedData("limit".to_string()));
        }
        let id: DatasetId = dataset_data.get("id").unwrap().parse().unwrap();
        let mut dataset = Dataset {
            id: id.clone(),
            limit: dataset_data.get("limit").unwrap().parse().unwrap(),
            count: conn.scard(Self::get_dataset_flight_data_key(&id))?,
            web3: None
        };
        match dataset_data.get("web3") {
            Some(serialized_web3) => {
                dataset.web3 = serde_json::from_str(&serialized_web3).unwrap() //TODO: manage this error in case something gets corrupted outside
            },
            None => ()
        };
        Ok(dataset)
    }

}

impl DeviceStorage for RedisStorage {
    fn new_device(&self, device: &Device) -> Result<()> {
        let device_key = Self::get_device_key(&device.id);
        let fields = Self::get_device_fields(device);
        //Note: The following is ok only in the assumption this is the only connection accessing Redis.
        let mut cnx_lock = self.conn.lock().unwrap();
        if cnx_lock.exists(device_key.clone())? {
            return Err(Error::AlreadyExists);
        }
        cnx_lock.hset_multiple(device_key, &fields)?;
        Ok(())
    }

    fn update_device(&self, device: &Device) -> Result<()> {
        if device.web3.is_none() {
            return Err(Error::NoOp);
        }
        let device_key = Self::get_device_key(&device.id);
        let mut cnx_lock = self.conn.lock().unwrap();
        if cnx_lock.exists(device_key.clone())? {
            let web3_str = serde_json::to_string(&device.web3).unwrap();
            cnx_lock.hset(device_key, "web3", web3_str)?;
            Ok(())
        } else {
            Err(Error::NotFound(Entity::Device))
        }
    }

    fn get_device(&self, id: &DeviceId) -> Result<Device> {
        let device_data: HashMap<String, String> = self.conn.lock().unwrap().hgetall(Self::get_device_key(id))?;
        if device_data.is_empty() {
            return Err(Error::NotFound(Entity::Device));
        }
        if !device_data.contains_key("public_key") {
            return Err(Error::Generic);
        }
        let mut device = Device {
            id: id.clone(),
            pk: device_data.get("public_key").unwrap().as_str().try_into().unwrap(),
            web3: None
        };
        match device_data.get("web3") {
            Some(serialized_web3) => {
                device.web3 = serde_json::from_str(&serialized_web3).unwrap() //TODO: manage this error in case something gets corrupted outside
            },
            None => ()
        };
        Ok(device)
    }
}

impl FlightDataStorage for RedisStorage {
    fn new_flight_data(&self, fd: &FlightData, device_id: &DeviceId) -> Result<Dataset> {
        let fd_key = Self::get_flight_data_key(&fd.id);
        let device_key = Self::get_device_key(device_id);
        let device_fd_key = Self::get_device_flight_data_key(device_id);
        let mut conn_lock = self.conn.lock().unwrap();
        let dataset_limit: u32 = conn_lock.hget(device_key.clone(), "dataset_limit")?;
        let fd_count: u32 = conn_lock.hincr(device_key.clone(), "flight_data_count", 1)?;
        let dataset = match fd_count % dataset_limit {
            1 => { //New dataset
                let mut dataset = Dataset::new(device_id.clone(), fd_count / dataset_limit, dataset_limit);
                dataset.count = 1;
                Self::no_lock_create_dataset(&mut conn_lock, &dataset, device_id)?;
                dataset
            },
            _ => { //Existing dataset
                let dataset_key = conn_lock.hget(device_key.clone(), "current_dataset")?;
                let mut dataset = Self::no_lock_get_dataset(&mut conn_lock, dataset_key)?;
                dataset.count = fd_count % dataset_limit;
                dataset.limit = dataset_limit;
                dataset
            }
        };
        // FlightData may arrive out of order so need a dedicated set to match them with the dataset
        conn_lock.sadd(Self::get_dataset_flight_data_key(&dataset.id), fd_key.clone())?;
        let fields = Self::get_flight_data_fields(fd, &dataset.id);
        conn_lock.hset_multiple(fd_key, &fields)?;
        conn_lock.zadd(device_fd_key, Self::get_flight_data_key(&fd.id), fd.timestamp)?;
        Ok(dataset)
    }
    
    fn get_flight_data(&self, id: &FlightDataId) -> Result<FlightData> {
        let fd_key = Self::get_flight_data_key(id);
        let mut conn_lock = self.conn.lock().unwrap();
        Self::no_lock_get_flight_data(&mut conn_lock, fd_key)
    }
    
    fn new_dataset(&self, limit: u32, device_id: &DeviceId) -> Result<Dataset> {
        todo!()
    }
    
    fn get_dataset(&self, id: &DatasetId) -> Result<Dataset> {
        Self::no_lock_get_dataset(&mut self.conn.lock().unwrap(), Self::get_dataset_key(id))
    }
    
    fn update_dataset_web3(&self, ds: &Dataset) -> Result<()> {
        let ds_key = Self::get_dataset_key(&ds.id);
        if ds.web3.is_none() {
            return Err(Error::NoOp);
        }
        let web3_str = serde_json::to_string(&ds.web3).unwrap();
        self.conn.lock().unwrap().hset(ds_key, "web3", web3_str)?;
        Ok(())
    }
    
    fn get_latest_dataset(&self, device_id: &DeviceId) -> Result<Option<Dataset>> {
        let mut conn_lock = self.conn.lock().unwrap();
        let dataset_key: String = conn_lock.hget(Self::get_device_key(device_id), "current_dataset")?;
        if dataset_key.is_empty() {
            return Err(Error::MalformedData("current_dataset".to_string()));
        }
        Ok(Some(Self::no_lock_get_dataset(&mut conn_lock, dataset_key)?))
    }
    
    fn get_dataset_flight_datas(&self, ds_id: &DatasetId) -> Result<Vec<FlightData>> {
        let mut conn_lock = self.conn.lock().unwrap();
        let fd_keys: Vec<String> = conn_lock.smembers(Self::get_dataset_flight_data_key(ds_id))?;
        let mut fds = Vec::new();
        for fd_key in fd_keys {
            fds.push(Self::no_lock_get_flight_data(&mut conn_lock, fd_key)?);
        }
        Ok(fds)
    }
    
    fn get_flight_data_dataset(&self, fd_id: &FlightDataId) -> Result<Dataset> {
        let mut conn_lock = self.conn.lock().unwrap();
        let dataset_key: String = conn_lock.hget(Self::get_flight_data_key(fd_id), "dataset")?;
        Self::no_lock_get_dataset(&mut conn_lock, dataset_key)
    }
}

impl From<RedisError> for Error {
    fn from(value: RedisError) -> Self {
        Error::Generic
    }
}

#[cfg(test)]
mod tests {

    use redis::Commands;

    use crate::state::entities::{Device, DeviceId, Dataset, FlightData, FlightDataId, PublicKey};
    use crate::storage::in_memory::InMemoryStorage;
    use crate::storage::errors::Error as StorageError;
    use crate::storage::redis::RedisStorage;
    use crate::storage::storage::{ DeviceStorage, FlightDataStorage };
    use crate::web3::traits::Web3Info;

    fn get_storage_instance() -> RedisStorage {
        RedisStorage::new("redis://localhost:6379").unwrap()
    }

    #[test]
    fn test_new_device() {
        let storage = get_storage_instance();
        let device = Device::test_instance();
        assert!(storage.new_device(&device).is_ok());
        let device_exists: bool = storage.conn.lock().unwrap().exists(RedisStorage::get_device_key(&device.id)).unwrap();
        assert!(device_exists);
    }

    #[test]
    fn test_new_device_duplicate() {
        let storage = get_storage_instance();
        let device = Device::test_instance();
        assert!(storage.new_device(&device).is_ok());
        match storage.new_device(&device) {
            Ok(_) => panic!("Duplicated Device was added instead of being rejected"),
            Err(err) => assert!(err == StorageError::AlreadyExists)
        }
    }

    #[test]
    fn test_new_device_and_get() {
        let storage = get_storage_instance();
        let device = Device::test_instance();
        assert!(storage.new_device(&device).is_ok(), "Could not add device");
        let gotten_device = storage.get_device(&device.id);
        assert!(gotten_device.is_ok(), "Could not get device");
        assert_eq!(device, gotten_device.unwrap(), "New device and gotten one are different");
    }

    #[test]
    fn test_device_update_and_get() {
        let storage = get_storage_instance();
        let mut device = Device::test_instance();
        let new_device_result = storage.new_device(&device);
        assert!(new_device_result.is_ok(), "Could not create a Device");
        device.web3 = Some(Web3Info::test_instance_no_merkle());
        assert!(storage.update_device(&device).is_ok(), "Could not update the Device");
        let gotten_device = storage.get_device(&device.id);
        assert!(gotten_device.is_ok(), "Could not get the Device");
        assert_eq!(device, gotten_device.unwrap(), "Updated Device and the gotten one are different")
    }

    #[test]
    fn test_new_flight_data() {
        let storage = get_storage_instance();
        let device = Device::test_instance();
        // Creates a new test device
        let new_device_result = storage.new_device(&device);
        assert!(new_device_result.is_ok(), "Could not create a Device");
        
        let fd = FlightData::test_instance(&device.id);
        let new_fd_result = storage.new_flight_data(&fd, &device.id);
        assert!(new_fd_result.is_ok(), "Could not create a FlightData: {:?}", new_fd_result.err().unwrap());
        
        let gotten_fd = storage.get_flight_data(&fd.id);
        assert!(gotten_fd.is_ok(), "Could not get the FlightData: {:?}", gotten_fd.err().unwrap());
        assert_eq!(fd, gotten_fd.unwrap(), "New FlightData and the gotten one are different");
        
        let dataset = new_fd_result.unwrap();
        assert_eq!(dataset.count, 1, "The count of the dataset is not 1");
        assert_eq!(dataset.limit, 10, "The limit of the dataset is not 10");
        assert!(dataset.web3.is_none(), "The dataset has a web3 info while it supposed not");

        let latest_dataset = storage.get_latest_dataset(&device.id);
        assert!(latest_dataset.is_ok(), "Could not get the latest Dataset");
        assert_eq!(Some(dataset.clone()), latest_dataset.unwrap(), "The latest Dataset is not the expected one");

        let gotten_dataset = storage.get_flight_data_dataset(&fd.id);
        assert!(gotten_dataset.is_ok(), "Could not get the Dataset from the FlightData");
        assert_eq!(dataset, gotten_dataset.unwrap(), "New Dataset and the gotten one are different");

        let gotten_dataset = storage.get_dataset(&dataset.id);
        assert!(gotten_dataset.is_ok(), "Could not get the Dataset from the Dataset Id");
        assert_eq!(dataset, gotten_dataset.unwrap(), "New Dataset and the gotten one are different");

        let gotten_fds = storage.get_dataset_flight_datas(&dataset.id);
        assert!(gotten_fds.is_ok(), "Could not get the FlightDatas from the Dataset");
        assert_eq!(vec![fd], gotten_fds.unwrap(), "New FlightData and the gotten one are different");
    }
}