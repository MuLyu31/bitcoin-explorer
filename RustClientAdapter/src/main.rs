mod bitcoin_api;
mod bitcoin_rpc;
mod config;
mod data_provider;
mod db;
mod metrics;
mod server;

use config::Config;
use data_provider::BitcoinDataProvider;
use db::connect_to_postgres;
use db::connect_to_postgres_with_retry;
use dotenv::dotenv;
use log::error;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use std::panic;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let config = Config::from_env();
    let db = Arc::new(connect_to_postgres_with_retry(&config.db_config).await?);
    let db_clone = db.clone();

    let bitcoin_provider = BitcoinDataProvider::new(config.use_api);

    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::start_server(db_clone).await {
            error!("Server error: {:?}", e);
        }
    });

    // Main loop for processing metrics
    loop {
        if let Err(e) = metrics::process_and_store_metrics(&bitcoin_provider, db.clone()).await {
            error!("Error processing metrics: {}", e);
        }
        sleep(Duration::from_secs(60)).await;
    }

    // This line will never be reached in the current setup,
    // but it's good practice to include it
    if let Err(e) = server_handle.await {
        error!("Server task error: {:?}", e);
    }

    Ok(())
}
