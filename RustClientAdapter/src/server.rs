use axum::{
    extract::Extension,
    response::Json,
    routing::get,
    Router,
    body::Body,
};
use http::Method;
use http::header::CONTENT_TYPE;
use serde::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_postgres::Client;
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize)]
struct Transaction {
    txid: String,
    block_height: i32,
}

#[derive(Serialize)]
struct BlockchainMetric {
    id: i32,
    block_height: i32,
    difficulty: f64,
    connection_count: i32,
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
        .query("SELECT id, timestamp, block_height, difficulty::float8, connection_count FROM blockchain_metrics ORDER BY timestamp DESC LIMIT 100", &[])
        .await
        .expect("Failed to execute query");
    let metrics: Vec<BlockchainMetric> = rows
        .into_iter()
        .map(|row| BlockchainMetric {
            id: row.get("id"),
            block_height: row.get("block_height"),
            difficulty: row.get("difficulty"),
            connection_count: row.get("connection_count"),
        })
        .collect();
    Json(metrics)
}
pub async fn start_server(client: Arc<Client>) {
    // Configure CORS
    let cors = CorsLayer::new()
        // Allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET])
        // Allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/transactions", get(get_transactions))
        .route("/blockchain_metrics", get(get_blockchain_metrics))
        .layer(Extension(client))
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Server starting on http://localhost:3001");
    axum::serve(listener, app).await.unwrap();
}
