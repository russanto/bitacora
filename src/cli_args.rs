use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CLIArgs {
    #[arg(short, long, default_value_t = String::from("http://localhost:8545"))]
    pub web3: String,
}