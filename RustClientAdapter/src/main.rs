use bitcoincore_rpc::bitcoin::Transaction as RpcTransaction;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::convert::TryInto;
use std::sync::Arc;
use tokio_postgres::{Config, NoTls};
use warp::{Filter, Reply};
use warp::http::StatusCode;
use warp::cors;
use std::convert::Infallible;

#[tokio::main]
async fn main() {
    // Connect to the Bitcoin Core node
    let rpc = Arc::new(
        Client::new(
            "http://127.0.0.1:8332",
            Auth::UserPass("myrpcuser".to_string(), "myrpcpassword".to_string()),
        )
            .unwrap(),
    );

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

    // Warp filter to get the current block height
    let cors = cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(vec!["GET", "POST"]);

    let block_height_route = {
        let rpc = Arc::clone(&rpc);
        warp::path!("block_height")
            .and(warp::get())
            .and_then(move || {
                let rpc = Arc::clone(&rpc);
                async move {
                    match rpc.get_block_count() {
                        Ok(height) => Ok::<_, Infallible>(warp::reply::json(&height).into_response()),
                        Err(e) => {
                            eprintln!("Error fetching block height: {:?}", e);
                            let err_msg = format!("Error fetching block height: {:?}", e);
                            Ok::<_, Infallible>(warp::reply::with_status(err_msg, StatusCode::INTERNAL_SERVER_ERROR).into_response())
                        }
                    }
                }
            })
            .with(cors)
    };

    // Spawn the loop that processes blocks and transactions in the background
    let rpc_clone = Arc::clone(&rpc);
    let client_ref = Arc::new(client); // Wrap the client in an Arc for shared ownership
    tokio::spawn(async move {
        // Get the latest block height processed
        let latest_block_height = get_latest_block_height_from_database(&client_ref).await;

        // Continuously fetch new blocks and transactions
        loop {
            let current_block_height = rpc_clone.get_block_count().unwrap() as i32;

            // Fetch transactions for each new block
            for block_height in (latest_block_height + 1)..=current_block_height {
                let block_hash = rpc_clone
                    .get_block_hash(block_height.try_into().unwrap())
                    .unwrap();
                let block = rpc_clone.get_block(&block_hash).unwrap();

                // Process transactions in the block
                for tx in block.txdata.iter() {
                    // Extract transaction details and insert into PostgreSQL database
                    process_transaction(&client_ref, tx, block_height).await;
                }

                // Update the latest processed block height in the database
                update_latest_block_height_in_database(&client_ref, block_height).await;
            }

            // Sleep for some time before checking for new blocks again
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    // Start the Warp server
    warp::serve(block_height_route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn process_transaction(client: &tokio_postgres::Client, tx: &RpcTransaction, block_height: i32) {
    // Extract transaction details and insert into PostgreSQL database
    let txid = tx.compute_txid();
    let fee = calculate_transaction_fee(tx).await; // Implement this function
    insert_transaction_into_database(client, &txid.to_string(), fee, block_height).await;
}

async fn calculate_transaction_fee(tx: &RpcTransaction) -> i64 {
    // Calculate transaction fee (dummy implementation)
    // In a real implementation, you need to fetch the previous transactions to get the input values
    let input_value: i64 = 0; // Replace with actual calculation
    let output_value: i64 = tx
        .output
        .iter()
        .map(|output| output.value.to_sat() as i64)
        .sum();

    input_value - output_value
}

async fn insert_transaction_into_database(client: &tokio_postgres::Client, txid: &str, fee: i64, block_height: i32) {
    // Insert transaction details into PostgreSQL database
    if let Err(e) = client
        .execute(
            "INSERT INTO transactions (txid, fee, block_height) VALUES ($1, $2, $3)",
            &[&txid, &fee, &block_height],
        )
        .await
    {
        eprintln!("Failed to insert transaction into database: {}", e);
    }
}

async fn get_latest_block_height_from_database(client: &tokio_postgres::Client) -> i32 {
    // Fetch and return the latest processed block height from the database
    let row = client
        .query_one(
            "SELECT COALESCE(MAX(block_height), 0) FROM transactions",
            &[],
        )
        .await
        .expect("Failed to fetch latest block height from database");

    row.get::<usize, i32>(0)
}

async fn update_latest_block_height_in_database(
    client: &tokio_postgres::Client,
    block_height: i32,
) {
    if let Err(e) = client
        .execute(
            "INSERT INTO block_heights (block_height) VALUES ($1) ON CONFLICT DO NOTHING",
            &[&block_height],
        )
        .await
    {
        eprintln!("Failed to update latest block height in database: {}", e);
    }
}