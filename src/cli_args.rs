use clap::Parser;

use crate::state::bitacora::DATASET_DEFAULT_LIMIT;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]

/// Struct representing the command-line arguments for the application.
pub struct CLIArgs {
    /// The web3 URL to connect to. Defaults to "http://localhost:8545".
    #[arg(long, default_value_t = String::from("http://localhost:8545"))]
    pub web3: String,

    /// The base path wher to find contracts source code.
    #[arg(long)]
    pub contracts_base: String,

    /// The address of the contract. Optional. If not provided a new one will be created.
    #[arg(long)]
    pub contract_address: Option<String>,

    /// The private key to use for signing transactions.
    #[arg(short, long)]
    pub private_key: String,

    /// The maximum number of flight datas in a datasets. Defaults to DATASET_DEFAULT_LIMIT.
    #[arg(short, long, default_value_t = DATASET_DEFAULT_LIMIT)]
    pub dataset_limit: u32,

    /// The Redis URL to connect to. Defaults to "redis://localhost:6379".
    #[arg(short, long, default_value_t = String::from("redis://localhost:6379"))]
    pub redis: String,

    // /// Whether to use in-memory storage instead of Redis.
    // #[arg(long)]
    // pub in_memory: bool,
}