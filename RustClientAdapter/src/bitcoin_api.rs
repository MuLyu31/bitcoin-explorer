use reqwest::Client;
use std::cell::RefCell;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const BASE_URL: &str = "https://blockchain.info/q";
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
        let response = self.make_request("/getdifficulty").await?;
        Ok(response.parse()?)
    }

    pub async fn get_block_count(&self) -> Result<i64, Box<dyn Error>> {
        let response = self.make_request("/getblockcount").await?;
        Ok(response.parse()?)
    }

    pub async fn get_latest_hash(&self) -> Result<String, Box<dyn Error>> {
        self.make_request("/latesthash").await
    }

    // ... other methods ...
}
