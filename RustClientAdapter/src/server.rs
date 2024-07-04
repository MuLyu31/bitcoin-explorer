use axum::{
    extract::Extension,
    response::Json,
    routing::get,
    Router,
};
use http::Method;
use serde::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_postgres::Client;
use tower_http::cors::{Any, CorsLayer};
use std::error::Error;
use log::{error, info};

#[derive(Serialize)]
struct Transaction {
    txid: String,
    block_height: i32,
}

#[derive(Serialize)]
struct BlockchainMetric {
    id: i32,
    block_height: i64,
    difficulty: Option<f64>,
    connection_count: Option<i32>,
    tx_count: Option<i32>,
    block_size: Option<i32>,
    block_timestamp: Option<i64>,
    block_hash: Option<String>,
}
async fn get_transactions(Extension(client): Extension<Arc<Client>>) -> Json<Vec<Transaction>> {
    let rows = client
        .query("SELECT txid, block_height FROM transactions", &[])
        .await
        .expect("Failed to execute query");

    let transactions: Vec<Transaction> = rows
        .into_iter()
        .map(|row| Transaction {
            txid: row.get("txid"),
            block_height: row.get("block_height"),
        })
        .collect();

    Json(transactions)
}
async fn get_blockchain_metrics(
    Extension(client): Extension<Arc<Client>>,
) -> Json<Vec<BlockchainMetric>> {
    let rows = client
        .query("SELECT id, timestamp, block_height, difficulty::float8, connection_count, tx_count, block_size, block_timestamp, block_hash FROM blockchain_metrics ORDER BY block_height DESC LIMIT 100", &[])
        .await
        .expect("Failed to execute query");
    let metrics: Vec<BlockchainMetric> = rows
        .into_iter()
        .map(|row| BlockchainMetric {
            id: row.get("id"),
            block_height: row.get("block_height"),
            difficulty: row.get("difficulty"),
            connection_count: row.get("connection_count"),
            tx_count: row.get("tx_count"),
            block_size: row.get("block_size"),
            block_timestamp: row.get("block_timestamp"),
            block_hash: row.get("block_hash"),
        })
        .collect();
    Json(metrics)
}
pub async fn start_server(client: Arc<Client>) -> Result<(), Box<dyn Error>> {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/transactions", get(get_transactions))
        .route("/blockchain_metrics", get(get_blockchain_metrics))
        .layer(Extension(client))
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:3001").await
        .map_err(|e| {
            error!("Failed to bind to address: {:?}", e);
            e
        })?;

    info!("Server starting on http://localhost:3001");

    axum::serve(listener, app).await
        .map_err(|e| {
            error!("Server error: {:?}", e);
            e.into()
        })
}
