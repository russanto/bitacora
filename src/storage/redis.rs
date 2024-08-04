use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use base64::{engine::general_purpose::STANDARD, Engine as _};

use redis::{Commands, RedisError, RedisResult, ToRedisArgs};
use serde::{Deserialize, Serialize};
use serde_json;

use sha2::{Digest, Sha256};
use tracing_subscriber::registry::Data;

use crate::common::prelude::Bytes32;
use crate::configuration::BitacoraConfiguration as Conf;
use crate::state::entities::{
    Dataset, DatasetCounter, DatasetId, Device, DeviceId, Entity, FlightData, FlightDataId,
    Timestamp,
};

use super::errors::Error;
use super::storage::{DeviceStorage, FlightDataStorage, FullStorage};

type Result<T> = std::result::Result<T, Error>;

type RedisKey = String;

const PREFIX_DEVICE_KEY: &str = "device";

pub struct RedisStorage {
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
            ("dataset_count", "0".to_string()),
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
            (
                "localization",
                serde_json::to_string(&fd.localization).unwrap(),
            ),
            ("payload", STANDARD.encode(fd.payload.as_slice())),
            ("dataset", Self::get_dataset_key(ds_id)),
        ]
    }

    fn get_dataset_fields(ds: &Dataset, device_id: &DeviceId) -> Vec<(&'static str, String)> {
        let mut fields = vec![
            ("id", ds.id.clone()),
            ("device", Self::get_device_key(device_id)),
            ("limit", ds.limit.to_string()),
            ("count", ds.count.to_string()),
        ];
        if ds.web3.is_some() {
            fields.push(("web3", serde_json::to_string(&ds.web3).unwrap())) //TODO: replace with something more efficient
        }
        fields
    }

    fn no_lock_create_dataset(
        conn: &mut MutexGuard<redis::Connection>,
        dataset: &Dataset,
        device_id: &DeviceId,
    ) -> Result<()> {
        let dataset_key = Self::get_dataset_key(&dataset.id);
        let device_key = Self::get_device_key(device_id);
        conn.hset_multiple(
            dataset_key.clone(),
            &Self::get_dataset_fields(&dataset, device_id),
        )?;
        conn.hset(device_key, "current_dataset", dataset_key)?;
        Ok(())
    }

    fn no_lock_get_flight_data(
        conn: &mut MutexGuard<redis::Connection>,
        key: RedisKey,
    ) -> Result<FlightData> {
        let flight_data_data: HashMap<String, String> = conn.hgetall(key)?;
        if flight_data_data.is_empty() {
            return Err(Error::NotFound(Entity::FlightData));
        }
        if !flight_data_data.contains_key("id") {
            return Err(Error::MalformedData("id".to_string()));
        }
        let id: FlightDataId = match flight_data_data.get("id").unwrap().try_into() {
            Ok(id) => id,
            Err(_) => return Err(Error::MalformedData("id".to_string())),
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
            localization: serde_json::from_str(flight_data_data.get("localization").unwrap())
                .unwrap(),
            payload: STANDARD
                .decode(flight_data_data.get("payload").unwrap().as_bytes())
                .unwrap(),
        })
    }

    fn no_lock_get_dataset(
        conn: &mut MutexGuard<redis::Connection>,
        key: RedisKey,
    ) -> Result<Dataset> {
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
        if !dataset_data.contains_key("count") {
            return Err(Error::MalformedData("limit".to_string()));
        }
        let id: DatasetId = dataset_data.get("id").unwrap().parse().unwrap();
        let mut dataset = Dataset {
            id: id.clone(),
            limit: dataset_data.get("limit").unwrap().parse().unwrap(),
            count: dataset_data.get("count").unwrap().parse().unwrap(),
            web3: None,
        };
        match dataset_data.get("web3") {
            Some(serialized_web3) => {
                dataset.web3 = serde_json::from_str(&serialized_web3).unwrap() //TODO: manage this error in case something gets corrupted outside
            }
            None => (),
        };
        Ok(dataset)
    }
}

impl DeviceStorage for RedisStorage {
    fn new_device(&self, device: &Device, dataset_limit: u32) -> Result<()> {
        let device_key = Self::get_device_key(&device.id);
        let device_fields = Self::get_device_fields(device);
        let mut cnx_lock = self.conn.lock().unwrap();
        if cnx_lock.exists(device_key.clone())? {
            return Err(Error::AlreadyExists);
        }
        let dataset = Dataset::new(device.id.clone(), 0, dataset_limit);
        let dataset_key = Self::get_dataset_key(&dataset.id);
        let dataset_fields = Self::get_dataset_fields(&dataset, &device.id);
        // Should add a watch on the device key in case there are two concurrent requests
        redis::pipe()
            .atomic()
            .hset_multiple(device_key.clone(), &device_fields)
            .ignore()
            .hset_multiple(dataset_key.clone(), &dataset_fields)
            .ignore()
            .query(&mut *cnx_lock)?;
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
        let device_data: HashMap<String, String> = self
            .conn
            .lock()
            .unwrap()
            .hgetall(Self::get_device_key(id))?;
        if device_data.is_empty() {
            return Err(Error::NotFound(Entity::Device));
        }
        if !device_data.contains_key("public_key") {
            return Err(Error::Generic);
        }
        let mut device = Device {
            id: id.clone(),
            pk: device_data
                .get("public_key")
                .unwrap()
                .as_str()
                .try_into()
                .unwrap(),
            web3: None,
        };
        match device_data.get("web3") {
            Some(serialized_web3) => {
                device.web3 = serde_json::from_str(&serialized_web3).unwrap() //TODO: manage this error in case something gets corrupted outside
            }
            None => (),
        };
        Ok(device)
    }
}

impl FlightDataStorage for RedisStorage {
    fn new_flight_data(&self, fd: &FlightData, device_id: &DeviceId) -> Result<Dataset> {
        let script = include_str!("redis_script/new_flight_data.lua");

        let device_key = Self::get_device_key(device_id);
        let fd_key = Self::get_flight_data_key(&fd.id);
        let device_fd_key = Self::get_device_flight_data_key(device_id);

        let mut conn_lock = self.conn.lock().unwrap();
        let redis_script = redis::Script::new(script);
        let mut script_invocation = redis_script.key(device_key.clone());
        script_invocation
            .key(fd_key.clone())
            .key(device_fd_key.clone())
            .arg(fd.id.to_string())
            .arg(fd.signature.clone())
            .arg(fd.timestamp)
            .arg(serde_json::to_string(&fd.localization).unwrap())
            .arg(STANDARD.encode(fd.payload.as_slice()));
        let result: String = script_invocation.invoke(&mut *conn_lock)?;
        Ok(Self::no_lock_get_dataset(&mut conn_lock, result)?)
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
        let dataset_counter: DatasetCounter =
            conn_lock.hget(Self::get_device_key(device_id), "dataset_count")?;
        Ok(Some(Self::no_lock_get_dataset(
            &mut conn_lock,
            Self::get_dataset_key(&Dataset::dataset_id(device_id, dataset_counter)),
        )?))
    }

    fn get_dataset_flight_datas(&self, ds_id: &DatasetId) -> Result<Vec<FlightData>> {
        let mut conn_lock = self.conn.lock().unwrap();
        let fd_keys: Vec<String> =
            conn_lock.zrange(Self::get_dataset_flight_data_key(ds_id), 0, -1)?; // 0 to -1 is the whole range
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

impl FullStorage for RedisStorage {}

impl From<RedisError> for Error {
    fn from(value: RedisError) -> Self {
        Error::Generic
    }
}

#[cfg(test)]
mod tests {

    use std::borrow::Borrow;

    use redis::Commands;

    use crate::state::entities::{Dataset, Device, FlightData};
    use crate::storage::errors::Error as StorageError;
    use crate::storage::redis::RedisStorage;
    use crate::storage::storage::{DeviceStorage, FlightDataStorage};
    use crate::web3::traits::Web3Info;

    const DEFAULT_DATASET_LIMIT: u32 = 10;

    fn get_storage_instance() -> RedisStorage {
        RedisStorage::new("redis://localhost:6379").unwrap()
    }

    #[test]
    fn test_new_device() {
        let storage = get_storage_instance();
        let device = Device::test_instance();
        assert!(storage.new_device(&device, DEFAULT_DATASET_LIMIT).is_ok());
        let device_exists: bool = storage
            .conn
            .lock()
            .unwrap()
            .exists(RedisStorage::get_device_key(&device.id))
            .unwrap();
        assert!(device_exists);
        let dataset = storage
            .get_dataset(&Dataset::dataset_id(&device.id, 0))
            .expect("Could not get the associated dataset");
        assert_eq!(
            dataset.limit, DEFAULT_DATASET_LIMIT,
            "The dataset limit is not the expected one"
        );
        assert_eq!(
            dataset.count, 0,
            "The dataset count is not the expected one"
        );
    }

    #[test]
    fn test_new_device_duplicate() {
        let storage = get_storage_instance();
        let device = Device::test_instance();
        assert!(storage.new_device(&device, DEFAULT_DATASET_LIMIT).is_ok());
        match storage.new_device(&device, DEFAULT_DATASET_LIMIT) {
            Ok(_) => panic!("Duplicated Device was added instead of being rejected"),
            Err(err) => assert!(err == StorageError::AlreadyExists),
        }
    }

    #[test]
    fn test_new_device_and_get() {
        let storage = get_storage_instance();
        let device = Device::test_instance();
        assert!(
            storage.new_device(&device, DEFAULT_DATASET_LIMIT).is_ok(),
            "Could not add device"
        );
        let gotten_device = storage.get_device(&device.id);
        assert!(gotten_device.is_ok(), "Could not get device");
        assert_eq!(
            device,
            gotten_device.unwrap(),
            "New device and gotten one are different"
        );
    }

    #[test]
    fn test_device_update_and_get() {
        let storage = get_storage_instance();
        let mut device = Device::test_instance();
        let new_device_result = storage.new_device(&device, DEFAULT_DATASET_LIMIT);
        assert!(new_device_result.is_ok(), "Could not create a Device");
        device.web3 = Some(Web3Info::test_instance_no_merkle());
        assert!(
            storage.update_device(&device).is_ok(),
            "Could not update the Device"
        );
        let gotten_device = storage.get_device(&device.id);
        assert!(gotten_device.is_ok(), "Could not get the Device");
        assert_eq!(
            device,
            gotten_device.unwrap(),
            "Updated Device and the gotten one are different"
        )
    }

    #[test]
    fn test_new_flight_data() {
        let storage = get_storage_instance();
        let device = Device::test_instance();
        // Creates a new test device
        let new_device_result = storage.new_device(&device, DEFAULT_DATASET_LIMIT);
        assert!(new_device_result.is_ok(), "Could not create a Device");

        let fd = FlightData::test_instance(&device.id);
        let new_fd_result = storage.new_flight_data(&fd, &device.id);
        assert!(
            new_fd_result.is_ok(),
            "Could not create a FlightData: {:?}",
            new_fd_result.err().unwrap()
        );

        let gotten_fd = storage.get_flight_data(&fd.id);
        assert!(
            gotten_fd.is_ok(),
            "Could not get the FlightData: {:?}",
            gotten_fd.err().unwrap()
        );
        assert_eq!(
            fd,
            gotten_fd.unwrap(),
            "New FlightData and the gotten one are different"
        );

        let dataset = new_fd_result.unwrap();
        assert_eq!(dataset.count, 1, "The count of the dataset is not 1");
        assert_eq!(dataset.limit, 10, "The limit of the dataset is not 10");
        assert!(
            dataset.web3.is_none(),
            "The dataset has a web3 info while it supposed not"
        );

        let latest_dataset = storage.get_latest_dataset(&device.id);
        assert!(latest_dataset.is_ok(), "Could not get the latest Dataset");
        assert_eq!(
            Some(dataset.clone()),
            latest_dataset.unwrap(),
            "The latest Dataset is not the expected one"
        );

        let gotten_dataset = storage.get_flight_data_dataset(&fd.id);
        assert!(
            gotten_dataset.is_ok(),
            "Could not get the Dataset from the FlightData"
        );
        assert_eq!(
            dataset,
            gotten_dataset.unwrap(),
            "Gotten Flight Data Dataset is different from the one returned in creation"
        );

        let gotten_dataset = storage.get_dataset(&dataset.id);
        assert!(
            gotten_dataset.is_ok(),
            "Could not get the Dataset from the Dataset Id"
        );
        assert_eq!(
            dataset,
            gotten_dataset.unwrap(),
            "New Dataset and the gotten one are different"
        );

        let gotten_fds = storage.get_dataset_flight_datas(&dataset.id);
        assert!(
            gotten_fds.is_ok(),
            "Could not get the FlightDatas from the Dataset"
        );
        assert_eq!(
            vec![fd],
            gotten_fds.unwrap(),
            "New FlightData and the gotten one are different"
        );
    }

    #[test]
    fn test_new_dataset_on_dataset_limit() {
        // Test setup
        let storage = get_storage_instance();
        let device = Device::test_instance();
        // Creates a new test device
        let new_device_result = storage.new_device(&device, DEFAULT_DATASET_LIMIT);
        assert!(new_device_result.is_ok(), "Could not create a Device");

        let mut fds = FlightData::test_instance_list(DEFAULT_DATASET_LIMIT + 1, &device.id);
        let fd_in_new_dataset = fds.pop().unwrap();

        let mut current_dataset = storage.get_latest_dataset(&device.id).unwrap().unwrap();

        // Test body

        // Fulfil the initial Dataset
        for i in 0..fds.len() {
            let fd = fds.get(i).unwrap();
            let new_fd_result = storage.new_flight_data(fd, &device.id);
            assert!(
                new_fd_result.is_ok(),
                "Could not create {:}-th FlightData: {:?}",
                i,
                new_fd_result.err().unwrap()
            );

            let gotten_fd = storage.get_flight_data(&fd.id);
            assert!(
                gotten_fd.is_ok(),
                "Could not get {:}-th FlightData: {:?}",
                i,
                gotten_fd.err().unwrap()
            );
            assert_eq!(
                fd,
                &gotten_fd.unwrap(),
                "New FlightData and the gotten one are different at index {:}",
                i
            );

            current_dataset.count += 1;
            assert_eq!(
                current_dataset,
                new_fd_result.unwrap(),
                "The reference Dataset changed while it is not supposed at index {:}",
                i
            );
        }

        // Add one more Dataset to create a new one
        let new_dataset = storage.new_flight_data(&fd_in_new_dataset, &device.id);
        assert!(
            new_dataset.is_ok(),
            "Could not create the FlightData in the new Dataset: {:?}",
            new_dataset.err().unwrap()
        );
        let new_dataset = new_dataset.unwrap();

        // Check it is a new dataset
        assert_ne!(
            current_dataset.id, new_dataset.id,
            "The new Dataset is the same as the previous one"
        );

        // Check new Dataset was properly created
        assert_eq!(
            new_dataset.count, 1,
            "The count of the new dataset is not 1"
        );

        // Check the latest dataset is the new one
        let latest_dataset = storage.get_latest_dataset(&device.id);
        assert!(latest_dataset.is_ok(), "Could not get the latest Dataset");
        let latest_dataset = latest_dataset.unwrap();
        assert!(latest_dataset.is_some(), "The latest Dataset is None");
        assert_eq!(
            new_dataset,
            latest_dataset.unwrap(),
            "The latest Dataset is not the expected one"
        );

        // Check the FlightData is in the new Dataset
        let gotten_dataset = storage.get_flight_data_dataset(&fd_in_new_dataset.id);
        assert!(
            gotten_dataset.is_ok(),
            "Could not get the Dataset from the FlightData"
        );
        assert_eq!(
            new_dataset,
            gotten_dataset.unwrap(),
            "Gotten Flight Data Dataset is different from the one returned in creation"
        );

        // Check the Dataset can be retrieved
        let gotten_dataset = storage.get_dataset(&new_dataset.id);
        assert!(
            gotten_dataset.is_ok(),
            "Could not get the Dataset from the Dataset Id"
        );
        assert_eq!(
            new_dataset,
            gotten_dataset.unwrap(),
            "New Dataset and the gotten one are different"
        );

        // Check the FlightDatas can be retrieved from the new Dataset
        let gotten_fds = storage.get_dataset_flight_datas(&new_dataset.id);
        assert!(
            gotten_fds.is_ok(),
            "Could not get the FlightDatas from the Dataset"
        );
        assert_eq!(
            vec![fd_in_new_dataset],
            gotten_fds.unwrap(),
            "New FlightData and the gotten one are different"
        );
    }
}
