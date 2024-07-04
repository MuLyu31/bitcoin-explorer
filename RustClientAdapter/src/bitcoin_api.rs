use reqwest::Client;
use serde_json::Value;
use std::cell::RefCell;
use std::error::Error;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::time::sleep;

const BASE_URL: &str = "https://blockchain.info/";
const RATE_LIMIT: Duration = Duration::from_secs(10);
const MAX_RETRIES: u32 = 3;

#[derive(Error, Debug)]
pub enum BitcoinApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("API response missing expected field: {0}")]
    MissingField(String),
    #[error("API rate limit exceeded")]
    RateLimitExceeded,
    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,
}

pub struct BitcoinApi {
    client: Client,
    last_request: RefCell<Instant>,
}

impl BitcoinApi {
    pub fn new() -> Self {
        BitcoinApi {
            client: Client::new(),
            last_request: RefCell::new(Instant::now() - RATE_LIMIT),
        }
    }

    async fn make_request(&self, endpoint: &str) -> Result<String, BitcoinApiError> {
        let mut last_request = self.last_request.borrow_mut();
        let now = Instant::now();
        if now.duration_since(*last_request) < RATE_LIMIT {
            sleep(RATE_LIMIT - now.duration_since(*last_request)).await;
        }

        let url = format!("{}{}?cors=true", BASE_URL, endpoint);
        let response = self.client.get(&url).send().await?;
        if response.status().is_success() {
            let body = response.text().await?;
            *last_request = Instant::now();
            Ok(body)
        } else if response.status().as_u16() == 429 {
            Err(BitcoinApiError::RateLimitExceeded)
        } else {
            Err(BitcoinApiError::RequestFailed(
                response.error_for_status().unwrap_err(),
            ))
        }
    }

    pub async fn get_difficulty(&self) -> Result<f64, Box<dyn Error>> {
        let response = self.make_request("/q/getdifficulty").await?;
        Ok(response.parse()?)
    }

    pub async fn get_block_count(&self) -> Result<i64, Box<dyn Error>> {
        let response = self.make_request("/q/getblockcount").await?;
        Ok(response.parse()?)
    }

    pub async fn get_latest_hash(&self) -> Result<String, BitcoinApiError> {
        self.make_request("/q/latesthash").await
    }

    pub async fn get_block_info(&self, height: i64) -> Result<BlockInfo, BitcoinApiError> {
        let mut retries = 0;
        loop {
            match self.attempt_get_block_info(height).await {
                Ok(block_info) => return Ok(block_info),
                Err(e) => {
                    if retries >= MAX_RETRIES {
                        log::error!("Max retries exceeded for block height {}: {:?}", height, e);
                        return Err(BitcoinApiError::MaxRetriesExceeded);
                    }
                    log::warn!(
                        "Failed to fetch block info for height {}, retrying: {:?}",
                        height,
                        e
                    );
                    retries += 1;
                    sleep(Duration::from_secs(2u64.pow(retries))).await; // Exponential backoff
                }
            }
        }
    }
    pub async fn get_mempool_info(&self) -> Result<Value, Box<dyn Error>> {
        let response = self
            .make_request("/unconfirmed-transactions?format=json")
            .await?;
        let json: Value = serde_json::from_str(&response)?;
        Ok(json)
    }
    async fn attempt_get_block_info(&self, height: i64) -> Result<BlockInfo, BitcoinApiError> {
        let response = self
            .make_request(&format!("/block-height/{}?format=json", height))
            .await?;

        let json: Value = serde_json::from_str(&response)?;
        let block = &json["blocks"]
            .as_array()
            .ok_or_else(|| BitcoinApiError::MissingField("blocks array".to_string()))?
            .first()
            .ok_or_else(|| BitcoinApiError::MissingField("first block".to_string()))?;

        Ok(BlockInfo {
            height,
            hash: block["hash"]
                .as_str()
                .ok_or_else(|| BitcoinApiError::MissingField("hash".to_string()))?
                .to_string(),
            time: block["time"]
                .as_i64()
                .ok_or_else(|| BitcoinApiError::MissingField("time".to_string()))?,
            n_tx: block["n_tx"]
                .as_u64()
                .ok_or_else(|| BitcoinApiError::MissingField("n_tx".to_string()))?
                as u32,
            size: block["size"]
                .as_u64()
                .ok_or_else(|| BitcoinApiError::MissingField("size".to_string()))?
                as u32,
        })
    }
}

#[derive(Debug)]
pub struct BlockInfo {
    pub height: i64,
    pub hash: String,
    pub time: i64,
    pub n_tx: u32,
    pub size: u32,
}
