mod tests {

    use p256::ecdsa::{SigningKey, VerifyingKey};
    use p256::EncodedPoint;
    use rand::rngs::OsRng;

    use crate::state::entities::DeviceId;
    use crate::{state::{entities::{Device, PublicKey, FlightData, LocalizationPoint, FlightDataId, Dataset}, bitacora::{Bitacora, DATASET_DEFAULT_LIMIT}}, storage::in_memory::InMemoryStorage, web3::stub::EthereumStub};

    fn generate_p256_key_pair() -> (SigningKey, VerifyingKey) {
        // Generate a private key (for signing) using OsRng for randomness
        let signing_key = SigningKey::random(&mut OsRng);
    
        // Derive the corresponding public key from the private key
        let verifying_key = VerifyingKey::from(&signing_key);

        (signing_key, verifying_key)
    }

    impl Device {
        pub fn test_instance() -> Self {
            let (_, pk) = generate_p256_key_pair();
            let device_pk: PublicKey = pk.to_encoded_point(true).as_bytes()[1..].try_into().unwrap(); // true for compressed
            Device::from(device_pk)
        }
    }

    impl FlightData {
        pub fn test_instance(device_id: &DeviceId) -> Self {
            let timestamp = 1701305636123;
            FlightData {
                id: FlightDataId::new(timestamp, device_id),
                signature: String::new(),
                timestamp,
                localization: LocalizationPoint {
                    longitude: 14.425681,
                    latitude: 40.820948
                },
                payload: Vec::new()
            }
        }

        pub fn test_instance_list(n: u32) -> Vec<Self> {
            let device = Device::test_instance();
            let prototype = Self::test_instance(&device.id);
            let mut flight_datas: Vec<FlightData> = Vec::new();
            for i in 0..n {
                let mut fd = prototype.clone();
                fd.timestamp += 1000u64 * i as u64; // assume a FlightData object each second
                fd.localization.latitude += 0.01 * i as f64; // just to change data
                fd.localization.latitude += 0.01 * i as f64;
                fd.id = FlightDataId::new(fd.timestamp, &device.id);
                flight_datas.push(fd);
            }
            flight_datas
        }
    }

    fn new_bitacora_from_stubs() -> Bitacora<InMemoryStorage, EthereumStub> {
        let storage_in_memory = InMemoryStorage::default();
        let timestamper_stub = EthereumStub::default();

        Bitacora::new(storage_in_memory, timestamper_stub)
    }

    #[tokio::test]
    async fn test_basic_flow_on_in_memory_storage() { //TODO: Why it panics with two equal FlightData ?
        let mut device = Device::test_instance();
        let flight_datas = FlightData::test_instance_list(DATASET_DEFAULT_LIMIT*2);

        let bitacora = new_bitacora_from_stubs();
        
        // Create the device
        if bitacora.new_device(&mut device).await.is_err() {
            panic!("Failed adding a new Device");
        }

        // Submit FlightData objects
        let mut previous_dataset: Option<Dataset> = None;
        for i in 0..flight_datas.len() {
            let fd = flight_datas.get(i).unwrap();
            let ds = match bitacora.new_flight_data(&fd, &device.id).await {
                Ok(ds) => ds,
                Err(_) => panic!("Failed adding a new FlightData")
            };

            if ds.limit > ds.count {
                assert!(ds.web3.is_none());
            } else if ds.limit == ds.count {
                assert!(ds.web3.is_some());
            } else {
                panic!("Dataset limit exceeded by the FlightData count");
            }

            if i % DATASET_DEFAULT_LIMIT as usize != 0 {
                assert_eq!(previous_dataset.unwrap().id, ds.id, "New FlightData returned a different Dataset than expected");
            }
            previous_dataset = Some(ds);
        }

        
    }
}