mod tests {
    use crate::{state::{entities::{Device, PublicKey, FlightData, LocalizationPoint, FlightDataId, Dataset}, bitacora::{Bitacora, DATASET_DEFAULT_LIMIT}}, storage::in_memory::InMemoryStorage, web3::stub::EthereumStub};


    fn new_bitacora_from_stubs() -> Bitacora<InMemoryStorage, EthereumStub> {
        let storage_in_memory = InMemoryStorage::default();
        let timestamper_stub = EthereumStub::default();

        Bitacora::new(storage_in_memory, timestamper_stub)
    }

    #[tokio::test]
    async fn test_basic_flow_on_in_memory_storage() { //TODO: Why it panics with two equal FlightData ?
        let device_pk: PublicKey = "0x1234567890123456789012345678901234567890123456789012345678901234".try_into().unwrap();
        let mut device = Device::from(device_pk);

        let flight_data_prototype = FlightData {
            id: FlightDataId::default(),
            signature: String::new(),
            timestamp: 1701305636123,
            localization: LocalizationPoint {
                longitude: 14.425681,
                latitude: 40.820948
            },
            payload: String::new()
        };
        let mut flight_datas: Vec<FlightData> = Vec::new();

        for i in 0..DATASET_DEFAULT_LIMIT*2 {
            let mut fd = flight_data_prototype.clone();
            fd.timestamp += 1000u64 * i as u64; // assume a FlightData object each second
            fd.localization.latitude += 0.01 * i as f64; // just to change data
            fd.localization.latitude += 0.01 * i as f64;
            fd.id = FlightDataId::new(fd.timestamp, &device.id);
            flight_datas.push(fd);
        }

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