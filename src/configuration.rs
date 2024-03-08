use axum::http::Uri;
use ethers::types::Address;
use once_cell::sync::Lazy;
use std::sync::RwLock;

use crate::{cli_args::CLIArgs, web3::traits::Web3Error};

pub struct BitacoraConfiguration {
    pub web3: Web3Configuration,
    pub default_dataset_limit: u32,
    pub redis_connection_string: String,
}

impl BitacoraConfiguration {
    fn instance() -> &'static RwLock<Self> {
        static CONFIGURATION: Lazy<RwLock<BitacoraConfiguration>> =
            Lazy::new(|| RwLock::new(BitacoraConfiguration::default()));
        &CONFIGURATION
    }

    pub fn from_cli_args(args: CLIArgs) -> Result<(), BitacoraConfigurationError> {
        let mut configuration = Self::instance().write().unwrap();
        *configuration = args.try_into()?;
        Ok(())
    }

    pub fn get_default_dataset_limit() -> u32 {
        BitacoraConfiguration::instance()
            .read()
            .unwrap()
            .default_dataset_limit
    }

    pub fn get_web3_uri() -> Uri {
        BitacoraConfiguration::instance()
            .read()
            .unwrap()
            .web3
            .url
            .clone()
    }

    pub fn get_web3_contract_base_dir() -> String {
        BitacoraConfiguration::instance()
            .read()
            .unwrap()
            .web3
            .contracts_base_dir
            .clone()
    }

    pub fn get_web3_contract_address() -> Option<Address> {
        BitacoraConfiguration::instance()
            .read()
            .unwrap()
            .web3
            .address
            .clone()
    }

    pub fn get_web3_signer() -> Option<String> {
        BitacoraConfiguration::instance()
            .read()
            .unwrap()
            .web3
            .signer
            .clone()
    }

    pub fn get_redis_connection_string() -> String {
        BitacoraConfiguration::instance()
            .read()
            .unwrap()
            .redis_connection_string
            .clone()
    }
}

impl Default for BitacoraConfiguration {
    fn default() -> Self {
        BitacoraConfiguration {
            web3: Web3Configuration {
                url: String::from("http://localhost:8545").parse().unwrap(),
                address: None,
                signer: None,
                contracts_base_dir: String::from("."),
            },
            default_dataset_limit: 0,
            redis_connection_string: String::from("redis://localhost:6379"),
        }
    }
}

impl TryFrom<CLIArgs> for BitacoraConfiguration {
    type Error = BitacoraConfigurationError;

    fn try_from(value: CLIArgs) -> Result<Self, Self::Error> {
        Ok(BitacoraConfiguration {
            default_dataset_limit: value.dataset_limit,
            redis_connection_string: value.redis.clone(),
            web3: value.try_into()?,
        })
    }
}

#[derive(Debug)]
pub enum BitacoraConfigurationError {
    Web3(Web3Error),
}

impl From<Web3Error> for BitacoraConfigurationError {
    fn from(value: Web3Error) -> Self {
        BitacoraConfigurationError::Web3(value)
    }
}

pub struct Web3Configuration {
    pub url: Uri,
    pub address: Option<Address>,
    pub signer: Option<String>,
    pub contracts_base_dir: String,
}

impl TryFrom<CLIArgs> for Web3Configuration {
    type Error = Web3Error;

    fn try_from(value: CLIArgs) -> Result<Self, Self::Error> {
        let address = match value.contract_address {
            Some(address_string) => match address_string.parse() {
                Ok(address) => Some(address),
                Err(_) => return Err(Web3Error::BadInputData(String::from("Contract Address"))),
            },
            None => None,
        };
        let uri: Uri = match value.web3.parse() {
            Ok(uri) => uri,
            Err(_) => return Err(Web3Error::BadInputData(String::from("Web3 Endpoint"))),
        };
        Ok(Web3Configuration {
            url: uri,
            address: address,
            signer: Some(value.private_key),
            contracts_base_dir: value.contracts_base,
        })
    }
}
