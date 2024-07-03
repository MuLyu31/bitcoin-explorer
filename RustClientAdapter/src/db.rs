use crate::config::DatabaseConfig;
use std::time::Duration;
use tokio::time::sleep;
use tokio_postgres::{Client, NoTls};

pub async fn connect_to_postgres(config: &DatabaseConfig) -> Result<Client, tokio_postgres::Error> {
    let connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        config.host, config.port, config.username, config.password, config.database_name
    );
    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    Ok(client)
}

pub async fn connect_to_postgres_with_retry(
    config: &DatabaseConfig,
) -> Result<Client, tokio_postgres::Error> {
    let mut retries = 5;
    let mut delay = Duration::from_secs(1);

    loop {
        match connect_to_postgres(config).await {
            Ok(client) => return Ok(client),
            Err(e) => {
                if retries == 0 {
                    eprintln!(
                        "Failed to connect to database after all retries. Last error: {}",
                        e
                    );
                    return Err(e);
                }
                eprintln!(
                    "Failed to connect to database: {}. Retrying in {:?}...",
                    e, delay
                );
                sleep(delay).await;
                retries -= 1;
                delay *= 2; // Exponential backoff
            }
        }
    }
}

pub async fn initialize_database(client: &Client) -> Result<(), tokio_postgres::Error> {
    const CREATE_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS blockchain_metrics (
        id SERIAL PRIMARY KEY,
        timestamp TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        block_height BIGINT NOT NULL,
        difficulty TEXT NOT NULL,
        connection_count INTEGER,
        tx_count INTEGER,
        block_size INTEGER,
        block_timestamp BIGINT,
        block_hash TEXT
    )";

    client.execute(CREATE_TABLE_SQL, &[]).await?;
    println!("Database initialized successfully");
    Ok(())
}

pub async fn setup_database(config: &DatabaseConfig) -> Result<Client, Box<dyn std::error::Error>> {
    let client = connect_to_postgres_with_retry(config).await?;
    initialize_database(&client).await?;
    Ok(client)
}
