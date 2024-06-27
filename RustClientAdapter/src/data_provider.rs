use crate::bitcoin_api::BitcoinApi;
use crate::bitcoin_rpc::{
    connect_to_bitcoin_rpc, get_block_count, get_block_info, get_connection_count, get_difficulty,
    RpcClient,
};

pub enum DataSource {
    RPC(RpcClient),
    API(BitcoinApi),
}

pub struct BlockInfo {
    pub height: i64,
    pub hash: String,
    pub timestamp: i64,
    pub tx_count: u32,
    pub size: u32,
}

pub struct BitcoinDataProvider {
    source: DataSource,
}

impl BitcoinDataProvider {
    pub fn new(use_api: bool) -> Self {
        let source = if use_api {
            DataSource::API(BitcoinApi::new())
        } else {
            DataSource::RPC(connect_to_bitcoin_rpc())
        };
        BitcoinDataProvider { source }
    }

    pub async fn get_block_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        match &self.source {
            DataSource::RPC(rpc) => Ok(i64::from(get_block_count(rpc)?)),
            DataSource::API(api) => api.get_block_count().await,
        }
    }

    pub async fn get_difficulty(&self) -> Result<f64, Box<dyn std::error::Error>> {
        match &self.source {
            DataSource::RPC(rpc) => Ok(get_difficulty(rpc)?),
            DataSource::API(api) => api.get_difficulty().await,
        }
    }

    pub async fn get_connection_count(&self) -> Result<u64, Box<dyn std::error::Error>> {
        match &self.source {
            DataSource::RPC(rpc) => Ok(get_connection_count(rpc)?),
            DataSource::API(_) => Ok(0), // API doesn't provide connection count, return 0
        }
    }

    pub async fn get_block_info(
        &self,
        height: i64,
    ) -> Result<BlockInfo, Box<dyn std::error::Error>> {
        match &self.source {
            DataSource::RPC(rpc) => {
                let info = get_block_info(rpc, height)?;
                Ok(BlockInfo {
                    height,
                    hash: info.hash,
                    timestamp: info.time,
                    tx_count: info.n_tx,
                    size: info.size,
                })
            }
            DataSource::API(api) => {
                // Fallback implementation for API
                // This could be replaced with actual API calls if they become available
                Ok(BlockInfo {
                    height,
                    hash: format!("dummy_hash_for_block_{}", height),
                    timestamp: chrono::Utc::now().timestamp(),
                    tx_count: 0,
                    size: 0,
                })
            }
        }
    }
}
