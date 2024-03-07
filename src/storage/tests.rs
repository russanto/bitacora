#[cfg(test)]
mod tests {

    use crate::state::entities::{Device, DeviceId, Dataset, FlightData, FlightDataId, PublicKey};
    use crate::storage::in_memory::InMemoryStorage;
    use crate::storage::errors::Error as StorageError;
    use crate::storage::storage::{ DeviceStorage, FlightDataStorage };

    const DEFAULT_DATASET_LIMIT: u32 = 10;

    #[test]
    fn test_new_device() {
        let storage: InMemoryStorage = InMemoryStorage::default();
        let device = Device::test_instance();
        assert!(storage.new_device(&device, DEFAULT_DATASET_LIMIT).is_ok());
    }

    #[test]
    fn test_new_device_duplicate() {
        let storage: InMemoryStorage = InMemoryStorage::default();
        let device = Device::test_instance();
        assert!(storage.new_device(&device, DEFAULT_DATASET_LIMIT).is_ok());
        match storage.new_device(&device, DEFAULT_DATASET_LIMIT) {
            Ok(_) => panic!("Duplicated Device was added instead of being rejected"),
            Err(err) => assert!(err == StorageError::AlreadyExists)
        }
    }
}