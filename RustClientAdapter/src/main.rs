use bitcoincore_rpc::{Auth, Client, RpcApi};
use tokio_postgres::{Config, NoTls};
use std::convert::TryInto;
use bitcoincore_rpc::bitcoin::Transaction as RpcTransaction;

#[tokio::main]
async fn main() {
    // Connect to the Bitcoin Core node
    let rpc = Client::new("http://127.0.0.1:8332", Auth::UserPass("myrpcuser".to_string(), "myrpcpassword".to_string())).unwrap();
    // Set up PostgreSQL connection
    let (client, _connection) = Config::new()
        .user("postgres")
        .password("1234")
        .host("localhost")
        .port(5432)
        .dbname("bitcoin_explorer")
        .connect(NoTls)
        .await
        .expect("Failed to connect to PostgreSQL.");

    // The connection object should be run in the background.
    tokio::spawn(async move {
        if let Err(e) = _connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    // Get the latest block height processed
    let latest_block_height = get_latest_block_height_from_database(&client).await;

    // Continuously fetch new blocks and transactions
    loop {
        let current_block_height = rpc.get_block_count().unwrap() as i32;

        // Fetch transactions for each new block
        for block_height in (latest_block_height + 1)..=current_block_height {
            let block_hash = rpc.get_block_hash(block_height.try_into().unwrap()).unwrap();
            let block = rpc.get_block(&block_hash).unwrap();

            // Process transactions in the block
            for tx in block.txdata.iter() {
                // Extract transaction details and insert into PostgreSQL database
                process_transaction(&client, tx).await; // Insert client as parameter
            }
        }

        // Update the latest processed block height in the database
        update_latest_block_height_in_database(&client, current_block_height).await;

        // Sleep for some time before checking for new blocks again
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn process_transaction(client: &tokio_postgres::Client, tx: &RpcTransaction) {
    // Extract transaction details and insert into PostgreSQL database
    let txid = tx.compute_txid();
    let fee = calculate_transaction_fee(tx).await; // Implement this function
    insert_transaction_into_database(client, &txid.to_string(), fee).await;
}

async fn calculate_transaction_fee(tx: &RpcTransaction) -> i64 {
    // Calculate transaction fee (dummy implementation)
    // In a real implementation, you need to fetch the previous transactions to get the input values
    let input_value: i64 = 0; // Replace with actual calculation
    let output_value: i64 = tx.output.iter().map(|output| output.value.to_sat() as i64).sum();
    
    input_value - output_value
}

async fn insert_transaction_into_database(client: &tokio_postgres::Client, txid: &str, fee: i64) {
    // Insert transaction details into PostgreSQL database
    if let Err(e) = client
        .execute(
            "INSERT INTO transactions (txid, fee) VALUES ($1, $2)",
            &[&txid, &fee],
        )
        .await
    {
        eprintln!("Failed to insert transaction into database: {}", e);
    }
}

async fn get_latest_block_height_from_database(client: &tokio_postgres::Client) -> i32 {
    // Fetch and return the latest processed block height from the database
    let row = client
        .query_one("SELECT COALESCE(MAX(block_height), 0) FROM transactions", &[])
        .await
        .expect("Failed to fetch latest block height from database");

    row.get::<usize, i32>(0)
}

async fn update_latest_block_height_in_database(client: &tokio_postgres::Client, block_height: i32) {
    // Update the latest processed block height in the database
    if let Err(e) = client
        .execute(
            "INSERT INTO latest_block_height (id, block_height) VALUES (1, $1) ON CONFLICT (id) DO UPDATE SET block_height = EXCLUDED.block_height",
            &[&block_threshold],
        )
        .await
    {
        eprintln!("Failed to update latest block height in database: {}", e);
    }
}
