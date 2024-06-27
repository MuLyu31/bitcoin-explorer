use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::bitcoin::consensus::encode::serialize;
use std::sync::Arc;
use log::{info, error};

pub type RpcClient = Arc<Client>;

pub use bitcoincore_rpc::bitcoin::Transaction as RpcTransaction;

pub struct BlockInfo {
    pub height: i64,
    pub hash: String,
    pub time: i64,
    pub n_tx: u32,
    pub size: u32,
}

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

pub fn get_block_info(rpc: &RpcClient, height: i64) -> Result<BlockInfo, bitcoincore_rpc::Error> {
    let hash = get_block_hash(rpc, height as i32)?;
    let block = get_block(rpc, &hash)?;
    let size = serialize(&block).len() as u32;
    Ok(BlockInfo {
        height,
        hash: hash.to_string(),
        time: block.header.time as i64,
        n_tx: block.txdata.len() as u32,
        size,
    })
}
