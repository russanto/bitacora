use clap::Parser;

use crate::state::bitacora::DATASET_DEFAULT_LIMIT;

/// Simple program to greet a person
#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct CLIArgs {
    #[arg(long, default_value_t = String::from("http://localhost:8545"))]
    pub web3: String,
    #[arg(long)]
    pub contracts_base: String,
    #[arg(long)]
    pub contract_address: Option<String>,
    #[arg(short, long)]
    pub private_key: String,
    #[arg(short, long, default_value_t = DATASET_DEFAULT_LIMIT)]
    pub dataset_count: u32
}