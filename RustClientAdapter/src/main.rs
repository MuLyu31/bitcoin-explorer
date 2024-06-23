mod bitcoin_rpc;
mod db;
mod server;

use bitcoin_rpc::{
    connect_to_bitcoin_rpc, get_block, get_block_count, get_block_hash, get_difficulty,
    get_connection_count, RpcClient, RpcTransaction,
};
use db::{connect_to_postgres, insert_transaction, DatabaseConfig};
use dotenv::dotenv;
use log::{error, info};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    // Define the database configuration from environment variables.
    let db_config = DatabaseConfig::from_env();

    // Connect to Bitcoin Core RPC.
    let rpc = connect_to_bitcoin_rpc();

    // Connect to PostgreSQL.
    let db = connect_to_postgres(&db_config).await;
    // Clone the database connection to pass to the server
    let db_clone = db.clone();

    let table_name = "transactions";

    // Start the server in a separate task.
    tokio::spawn(async move {
        server::start_server(db_clone).await;
    });

    loop {
        process_latest_blocks(rpc.clone(), db.clone(), table_name).await;
        sleep(Duration::from_secs(60)).await;
    }
}

async fn process_latest_blocks(rpc: RpcClient, db: Arc<tokio_postgres::Client>, table_name: &str) {
    let block_count = match get_block_count(&rpc) {
        Ok(count) => {
            info!("Fetched Block count: {}", count);
            count
        }
        Err(e) => {
            error!("Failed to fetch block count: {}", e);
            return;
        }
    };
    // Fetch difficulty
    let difficulty = match get_difficulty(&rpc) {
        Ok(diff) => diff.to_string(),
        Err(e) => {
            error!("Failed to fetch difficulty: {}", e);
            return;
        }
    };

    let connection_count = match get_connection_count(&rpc) {
        Ok(count) => {
            info!("Fetched connection count: {}", count);
            count as i32
        },
        Err(e) => {
            error!("Failed to fetch connection count: {}", e);
            return;
        }
    };
    // Insert metrics
    if let Err(e) = insert_metrics(&db, block_count, difficulty, connection_count).await {
        error!("Failed to insert metrics: {}", e);
    }

    // Fetch and process the latest 10 blocks
    for block_height in (block_count - 9)..=block_count {
        let block_hash = match get_block_hash(&rpc, block_height) {
            Ok(hash) => hash,
            Err(e) => {
                error!("Error fetching block hash: {:?}", e);
                continue;
            }
        };

        let block = match get_block(&rpc, &block_hash) {
            Ok(block) => block,
            Err(e) => {
                error!("Error fetching block: {:?}", e);
                continue;
            }
        };

        // Do not need to record each transaction in the block.
        // for tx in block.txdata.iter() {
        //     process_transaction(&db, tx, block_height, table_name).await;
        // }
        insert_transaction(
            &db,
            &block.txdata[0].compute_txid().to_string(),
            block_height,
            table_name,
        )
        .await;
    }
}
async fn insert_metrics(client: &Arc<tokio_postgres::Client>, block_height: i32, difficulty: String, connection_count: i32) -> Result<(), tokio_postgres::Error> {
    info!("Inserting metrics - Block Height: {}, Difficulty: {}, Connection count: {}", block_height, difficulty, connection_count);
    client.execute(
        "INSERT INTO blockchain_metrics (block_height, difficulty, connection_count) 
         VALUES ($1, $2, $3)",
        &[&block_height, &difficulty, &connection_count],
    )
    .await?;
    Ok(())
}

#[allow(dead_code)]
async fn process_transaction(
    client: &Arc<tokio_postgres::Client>,
    tx: &RpcTransaction,

    block_height: i32,
    table_name: &str,
) {
    let txid = tx.compute_txid().to_string();
    insert_transaction(client, &txid, block_height, table_name).await;
}
