use crate::data_provider::BitcoinDataProvider;
use log::{error, info};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio_postgres::Client;

// Use an atomic integer to store the last known block count
static LAST_BLOCK_COUNT: AtomicI64 = AtomicI64::new(-1);

pub async fn process_and_store_metrics(
    provider: &BitcoinDataProvider,
    db: Arc<Client>,
) -> Result<(), Box<dyn std::error::Error>> {
    let block_count = match provider.get_block_count().await {
        Ok(count) => {
            info!("Fetched Block count: {}", count);
            count
        }
        Err(e) => {
            error!("Failed to fetch block count: {}", e);
            return Err(e.into());
        }
    };

    // Check if the block count has changed
    let last_count = LAST_BLOCK_COUNT.load(Ordering::Relaxed);
    if block_count > last_count {
        // Update the last known block count
        LAST_BLOCK_COUNT.store(block_count, Ordering::Relaxed);

        // Fetch additional metrics only if we're going to insert a new record
        let difficulty = match provider.get_difficulty().await {
            Ok(diff) => diff,
            Err(e) => {
                error!("Failed to fetch difficulty: {}", e);
                return Err(e.into());
            }
        };

        let connection_count = match provider.get_connection_count().await {
            Ok(count) => {
                info!("Fetched connection count: {}", count);
                count
            }
            Err(e) => {
                error!("Failed to fetch connection count: {}", e);
                return Err(e.into());
            }
        };

        // Insert new metrics into the database
        insert_metrics(&db, block_count as i32, difficulty, connection_count as i32).await?;
        info!("Inserted new metrics for block height: {}", block_count);
    } else {
        info!(
            "Block count unchanged ({}). Skipping metrics insert.",
            block_count
        );
    }

    Ok(())
}

async fn insert_metrics(
    client: &Arc<Client>,
    block_height: i32,
    difficulty: f64,
    connection_count: i32,
) -> Result<(), tokio_postgres::Error> {
    info!(
        "Inserting metrics - Block Height: {}, Difficulty: {}, Connection count: {}",
        block_height, difficulty, connection_count
    );
    // Convert difficulty to a String
    let difficulty_str = difficulty.to_string();
    client
        .execute(
            "INSERT INTO blockchain_metrics (block_height, difficulty, connection_count) 
         VALUES ($1, $2, $3)",
            &[&block_height, &difficulty_str, &connection_count],
        )
        .await?;
    Ok(())
}
