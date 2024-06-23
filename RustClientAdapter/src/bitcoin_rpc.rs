use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::sync::Arc;
use log::{info, error};

pub type RpcClient = Arc<Client>;

pub use bitcoincore_rpc::bitcoin::Transaction as RpcTransaction;

pub fn connect_to_bitcoin_rpc() -> RpcClient {
    Arc::new(
        Client::new(
            "http://127.0.0.1:8332",
            Auth::UserPass("myrpcuser".to_string(), "myrpcpassword".to_string()),
        )
        .unwrap(),
    )
}

pub fn get_block_count(rpc: &RpcClient) -> Result<i32, bitcoincore_rpc::Error> {
    rpc.get_block_count().map(|count| count as i32)
}

pub fn get_block_hash(rpc: &RpcClient, block_height: i32) -> Result<bitcoincore_rpc::bitcoin::BlockHash, bitcoincore_rpc::Error> {
    rpc.get_block_hash(block_height as u64)
}

pub fn get_block(rpc: &RpcClient, block_hash: &bitcoincore_rpc::bitcoin::BlockHash) -> Result<bitcoincore_rpc::bitcoin::Block, bitcoincore_rpc::Error> {
    rpc.get_block(block_hash)
}

pub fn get_difficulty(rpc: &RpcClient) -> Result<f64, bitcoincore_rpc::Error> {
    rpc.get_difficulty()
}

pub fn get_mempool_info(rpc: &RpcClient) -> Result<bitcoincore_rpc::jsonrpc::serde_json::Value, bitcoincore_rpc::Error> {
    rpc.call("getmempoolinfo", &[])
}
pub fn get_connection_count(rpc: &RpcClient) -> Result<u64, bitcoincore_rpc::Error> {
    rpc.call("getconnectioncount", &[])
}
