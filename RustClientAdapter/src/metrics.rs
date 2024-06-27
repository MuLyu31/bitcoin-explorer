use crate::data_provider::{BitcoinDataProvider, BlockInfo};
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

        let block_info = provider.get_block_info(block_count).await?;

        // Insert new metrics into the database
        insert_metrics(
            &db,
            block_count as i32,
            difficulty,
            connection_count as i32,
            &block_info,
        )
        .await?;
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
    block_info: &BlockInfo,
) -> Result<(), tokio_postgres::Error> {
    info!(
        "Inserting metrics - Block Height: {}, Difficulty: {}, Connection count: {}, Tx count: {}, Size: {}, Timestamp: {}",
        block_height, difficulty, connection_count, block_info.tx_count, block_info.size, block_info.timestamp
    );
    // Convert difficulty to a String
    let difficulty_str = difficulty.to_string();
    client
        .execute(
            "INSERT INTO blockchain_metrics (block_height, difficulty, connection_count, tx_count, block_size, block_timestamp, block_hash) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (block_height) 
             DO UPDATE SET 
                difficulty = EXCLUDED.difficulty,
                connection_count = EXCLUDED.connection_count,
                tx_count = EXCLUDED.tx_count,
                block_size = EXCLUDED.block_size,
                block_timestamp = EXCLUDED.block_timestamp,
                block_hash = EXCLUDED.block_hash",
            &[&block_height, &difficulty_str, &connection_count, &(block_info.tx_count as i32), &(block_info.size as i32), &block_info.timestamp, &block_info.hash],
        )
        .await?;
    Ok(())
}
