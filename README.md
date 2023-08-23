# Bitacora
Bitacora is the Blockchain API for the CertiFlight portal in the CertiFlight project. The "cuaderno de bitácora" is the old sailors logbook.
## Documentation
### Data Model
The API assumes the following data model. Graphs are rendered with mermaid.js

```mermaid
classDiagram
    class FlightData {
        timestamp
        localization
        signature
        payload
        signature_full
        validate()
    }

    class Device {
        id
        public_key
    }

    class Dataset {
        id
        merkle_root
        limit
        getProof(FlightData)
    }

    class DatasetStatus {
        <<enum>>
        Initialized
        Pending
        Sealed
        Submitted
    }

    class Web3Info {
        blockchain
        transaction
    }

    class Error {
        code
        message
        description
        Error: parent
    }

    FlightData --o Dataset
    FlightData --* Device
    Dataset --* Device
    DatasetStatus --* Dataset
    Web3Info -- Device
    Web3Info -- Dataset
```

### API definition
The API is defined as a REST API using JSON over HTTP. It generally passes data in the messages body while configurations and options are passed as query parameters.

The following list is a summary of the avialble resources and methods. A precise description with example requests and responses is available in the Postman collection.