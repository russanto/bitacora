use axum::{
    routing::{get, post},
    Router
};
use state::bitacora::Bitacora;
use web3::{ethereum::new_ethereum_timestamper_from_devnode, traits::Timestamper};

use std::net::SocketAddr;
use std::sync::Arc;

pub mod handlers;
pub mod state;
pub mod storage;
pub mod web3;

use handlers::{ get_dataset, get_device, get_flight_data, post_device, post_flight_data };
use storage::{in_memory::InMemoryStorage, storage::FullStorage};

type SharedBitacora<S: FullStorage, T: Timestamper> = Arc<Bitacora<S, T>>;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let (timestamper, _anvil) = new_ethereum_timestamper_from_devnode().await;

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
        // .route("/flight_data/:id", get(get_flight_data::handler::<SharedBitacora<InMemoryStorage, EthereumTimestamper<>>))
        // .route("/dataset/:id", get(get_dataset::handler::<SharedBitacora>))
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