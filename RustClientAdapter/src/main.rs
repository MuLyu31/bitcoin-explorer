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
use sqlx::PgPool;
use std::panic;
use std::process::Command;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

async fn initialize_database(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // First, try to create the table using sqlx
    let result = sqlx::query(
        "CREATE TABLE IF NOT EXISTS blockchain_metrics (
            id SERIAL PRIMARY KEY,
            timestamp TIMESTAMP NOT NULL,
            difficulty DOUBLE PRECISION,
            block_count INTEGER
        )",
    )
    .execute(pool)
    .await;

    // If sqlx query fails, fall back to psql
    if result.is_err() {
        println!("Falling back to psql for database initialization...");
        let output = Command::new("psql")
            .arg(std::env::var("DATABASE_URL")?)
            .arg("-f")
            .arg("/usr/local/bin/init.sql")
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to initialize database: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
    }

    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    initialize_database(&pool).await?;

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
