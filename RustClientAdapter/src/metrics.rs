use crate::data_provider::{BitcoinDataProvider, BlockInfo};
use log::{error, info};
use std::sync::Arc;
use tokio_postgres::Client;
use std::collections::VecDeque;
use tokio::time::{sleep, Duration};

const BLOCK_HISTORY_SIZE: i64 = 10;
const UPDATE_INTERVAL: Duration = Duration::from_secs(60); // 1 minute

pub async fn process_and_store_metrics(
    provider: &BitcoinDataProvider,
    db: Arc<Client>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_processed_height: i64 = -1;

    loop {
        let latest_block_height = match provider.get_block_count().await {
            Ok(count) => {
                info!("Fetched latest block height: {}", count);
                count
            }
            Err(e) => {
                error!("Failed to fetch block count: {}", e);
                return Err(e.into());
            }
        };

        if latest_block_height > last_processed_height {
            // Process new blocks
            let start_height = std::cmp::max(latest_block_height - BLOCK_HISTORY_SIZE + 1, last_processed_height + 1);
            for height in start_height..=latest_block_height {
                process_block(provider, &db, height).await?;
            }
            last_processed_height = latest_block_height;
        } else {
            info!("No new blocks. Waiting for updates...");
        }

        // Wait before checking for updates again
        sleep(UPDATE_INTERVAL).await;
    }
}

async fn process_block(
    provider: &BitcoinDataProvider,
    db: &Arc<Client>,
    height: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let block_info = match provider.get_block_info(height).await {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to fetch block info for height {}: {}", height, e);
            return Err(e.into());
        }
    };

    let difficulty = match provider.get_difficulty().await {
        Ok(diff) => diff,
        Err(e) => {
            error!("Failed to fetch difficulty: {}", e);
            return Err(e.into());
        }
    };

    let connection_count = match provider.get_connection_count().await {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to fetch connection count: {}", e);
            return Err(e.into());
        }
    };

    insert_metrics(
        db,
        height,
        difficulty,
        connection_count as i32,
        &block_info,
    )
    .await?;

    info!("Processed and inserted metrics for block height: {}", height);
    Ok(())
}

async fn insert_metrics(
    client: &Arc<Client>,
    block_height: i64,
    difficulty: f64,
    connection_count: i32,
    block_info: &BlockInfo,
) -> Result<(), tokio_postgres::Error> {
    info!(
        "Inserting metrics - Block Height: {}, Difficulty: {}, Connection count: {}, Tx count: {}, Size: {}, Timestamp: {}",
        block_height, difficulty, connection_count, block_info.tx_count, block_info.size, block_info.timestamp
    );
    
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
