use reqwest::Client;
use serde_json::Value;
use std::cell::RefCell;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const BASE_URL: &str = "https://blockchain.info/";
const RATE_LIMIT: Duration = Duration::from_secs(10);

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

    async fn make_request(&self, endpoint: &str) -> Result<String, Box<dyn Error>> {
        let mut last_request = self.last_request.borrow_mut();
        let now = Instant::now();
        if now.duration_since(*last_request) < RATE_LIMIT {
            sleep(RATE_LIMIT - now.duration_since(*last_request)).await;
        }

        let url = format!("{}{}?cors=true", BASE_URL, endpoint);
        let response = self.client.get(&url).send().await?.text().await?;
        *last_request = Instant::now();

        Ok(response)
    }

    pub async fn get_difficulty(&self) -> Result<f64, Box<dyn Error>> {
        let response = self.make_request("/q/getdifficulty").await?;
        Ok(response.parse()?)
    }

    pub async fn get_block_count(&self) -> Result<i64, Box<dyn Error>> {
        let response = self.make_request("/q/getblockcount").await?;
        Ok(response.parse()?)
    }

    pub async fn get_latest_hash(&self) -> Result<String, Box<dyn Error>> {
        self.make_request("/q/latesthash").await
    }

    pub async fn get_block_info(&self, height: i64) -> Result<BlockInfo, Box<dyn Error>> {
        let response = self
            .make_request(&format!("/block-height/{}?format=json", height))
            .await?;
        let json: Value = serde_json::from_str(&response)?;
        let block = &json["blocks"][0];

        Ok(BlockInfo {
            height,
            hash: block["hash"].as_str().unwrap().to_string(),
            time: block["time"].as_i64().unwrap(),
            n_tx: block["n_tx"].as_u64().unwrap() as u32,
            size: block["size"].as_u64().unwrap() as u32,
        })
    }
    pub async fn get_mempool_info(&self) -> Result<Value, Box<dyn Error>> {
        let response = self
            .make_request("/unconfirmed-transactions?format=json")
            .await?;
        let json: Value = serde_json::from_str(&response)?;
        Ok(json)
    }
}

pub struct BlockInfo {
    pub height: i64,
    pub hash: String,
    pub time: i64,
    pub n_tx: u32,
    pub size: u32,
}
