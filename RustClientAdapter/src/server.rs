use axum::{
    routing::get,
    Router,
    extract::Extension,
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio::net::TcpListener;

#[derive(Serialize)]
struct Transaction {
    txid: String,
    block_height: i32,
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

pub async fn start_server(client: Arc<Client>) {
    let app = Router::new()
        .route("/transactions", get(get_transactions))
        .layer(Extension(client));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();


    axum::serve(listener, app.into_make_service()).await.unwrap();
}
