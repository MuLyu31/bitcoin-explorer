use tokio_postgres::{Client, Config, NoTls};
use std::sync::Arc;
use std::env;
use dotenv::dotenv;

// Wrap the tokio_postgres::Client in an Arc to make it thread-safe.
pub type DbClient = Arc<Client>;

pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub dbname: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        // Load environment variables from .env file.
        dotenv().ok();
        Self {
            username: env::var("DB_USERNAME").expect("DB_USERNAME not set"),
            password: env::var("DB_PASSWORD").expect("DB_PASSWORD not set"),
            host: env::var("DB_HOST").expect("DB_HOST not set"),
            port: env::var("DB_PORT").expect("DB_PORT not set").parse().expect("DB_PORT is not a valid u16"),
            dbname: env::var("DB_NAME").expect("DB_NAME not set"),
        }
    }

    pub fn to_config(&self) -> Config {
        let mut config = Config::new();
        config.user(&self.username);
        config.password(&self.password);
        config.host(&self.host);
        config.port(self.port);
        config.dbname(&self.dbname);
        config
    }
}

pub async fn connect_to_postgres(db_config: &DatabaseConfig) -> DbClient {
    let (client, connection) = db_config.to_config()
        .connect(NoTls)
        .await
        .expect("Failed to connect to PostgreSQL.");

    // The connection object should be run in the background.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    Arc::new(client)
}

pub async fn insert_transaction(client: &DbClient, txid: &str, block_height: i32, table_name: &str) {
    let query = format!("INSERT INTO {} (txid, block_height) VALUES ($1, $2)", table_name);
    if let Err(e) = client
        .execute(&query, &[&txid, &block_height])
        .await
    {
        eprintln!("Failed to insert transaction into database: {}", e);
    }
}
