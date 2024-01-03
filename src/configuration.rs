use once_cell::sync::Lazy;
use std::sync::RwLock;

use crate::cli_args::CLIArgs;

pub struct Web3Configuration {
    pub url: String,
    pub address: Option<String>,
    pub signer: Option<String>,
    pub contracts_base_dir: String
}

pub struct BitacoraConfiguration {
    pub web3: Web3Configuration,
    pub dataset_default_count: u32
}

impl BitacoraConfiguration {
    pub fn instance() -> &'static RwLock<Self> {
        static CONFIGURATION: Lazy<RwLock<BitacoraConfiguration>> = Lazy::new(|| {
            RwLock::new(BitacoraConfiguration::default())
        });
        &CONFIGURATION
    }

    pub fn from_cli_args(args: &CLIArgs) -> &'static RwLock<Self> {
        let mut configuration = Self::instance().write().unwrap();
        *configuration = args.to_owned().into();
        Self::instance()
    }

    pub fn get_dataset_default_count() -> u32 {
        BitacoraConfiguration::instance().read().unwrap().dataset_default_count
    }

    pub fn get_web3_contract_base_dir() -> String {
        BitacoraConfiguration::instance().read().unwrap().web3.contracts_base_dir.clone()
    }

    pub fn get_web3_signer() -> Option<String> {
        BitacoraConfiguration::instance().read().unwrap().web3.signer.clone()
    }
}

impl Default for BitacoraConfiguration {
    fn default() -> Self {
        BitacoraConfiguration {
            web3: Web3Configuration {
                url: String::from("http://localhost:8545"),
                address: None,
                signer: None,
                contracts_base_dir: String::from(".")
            },
            dataset_default_count: 0
        }
    
    }
}

impl From<CLIArgs> for BitacoraConfiguration {
    fn from(args: CLIArgs) -> Self {
        BitacoraConfiguration {
            web3: Web3Configuration {
                url: args.web3,
                address: None,
                signer: Some(args.private_key),
                contracts_base_dir: args.contracts_base
            },
            dataset_default_count: args.dataset_count
        }
    }
}