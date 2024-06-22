use axum::{
    extract::Extension,
    response::IntoResponse,
    routing::get,
    Router,
};
use http::StatusCode;
use std::sync::Arc;
use tokio_postgres::Client as DbClient;
use crate::bitcoin_rpc::{RpcClient, get_block_count};

pub fn app(rpc: RpcClient, db: Arc<DbClient>) -> Router {
    Router::new()
        .route("/block_height", get(block_height_handler))
        .layer(Extension(rpc))
        .layer(Extension(db))
}

async fn block_height_handler(
    Extension(rpc): Extension<RpcClient>,
) -> impl IntoResponse {
    match get_block_count(&rpc) {
        Ok(height) => (StatusCode::OK, format!("{}", height)).into_response(),
        Err(e) => {
            eprintln!("Error fetching block height: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error fetching block height: {:?}", e)).into_response()
        }
    }
}
