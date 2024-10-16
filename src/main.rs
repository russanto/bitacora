use axum::{
    routing::{get, post},
    Router
};
use clap::Parser;
use state::bitacora::Bitacora;
use web3::{ethereum::new_ethereum_timestamper_from_url_with_sk, traits::Timestamper};

use std::net::SocketAddr;
use std::sync::Arc;

pub mod cli_args;
pub mod common;
pub mod configuration;
pub mod handlers;
pub mod state;
pub mod storage;
pub mod web3;

use handlers::{ get_dataset, get_device, get_flight_data, post_device, post_flight_data };
use storage::{in_memory::InMemoryStorage, storage::FullStorage};

type SharedBitacora<S, T> = Arc<Bitacora<S, T>>;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // cli params
    let args = cli_args::CLIArgs::parse();
    configuration::BitacoraConfiguration::from_cli_args(&args);

    let timestamper = new_ethereum_timestamper_from_url_with_sk(&args.web3, &args.private_key).await.unwrap();

    let shared_bitacora = Arc::new(
        Bitacora::new(
            InMemoryStorage::default(),
            timestamper
        )
    );

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/device", post(post_device::handler))
        .route("/device/:id", get(get_device::handler))
        // `POST /users` goes to `create_user`
        .route("/flight_data", post(post_flight_data::handler))
        .route("/flight_data/:id", get(get_flight_data::handler))
        .route("/dataset/:id", get(get_dataset::handler))
        .with_state(shared_bitacora);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}