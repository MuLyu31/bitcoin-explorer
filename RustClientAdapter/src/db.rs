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
    );

    DO $$ 
    BEGIN
        -- Add a unique constraint on block_height if it doesn't exist
        IF NOT EXISTS (
            SELECT 1 FROM pg_constraint 
            WHERE conname = 'unique_block_height' AND conrelid = 'blockchain_metrics'::regclass
        ) THEN
            ALTER TABLE blockchain_metrics ADD CONSTRAINT unique_block_height UNIQUE (block_height);
        END IF;

        -- Create indexes for faster queries
        CREATE INDEX IF NOT EXISTS idx_blockchain_metrics_timestamp ON blockchain_metrics (timestamp);
        CREATE INDEX IF NOT EXISTS idx_blockchain_metrics_block_height ON blockchain_metrics (block_height);
    END $$;

    -- Create a function to update the timestamp
    CREATE OR REPLACE FUNCTION update_timestamp()
    RETURNS TRIGGER AS $$
    BEGIN
        NEW.timestamp = CURRENT_TIMESTAMP;
        RETURN NEW;
    END;
    $$ LANGUAGE plpgsql;

    -- Create a trigger to automatically update the timestamp
    DROP TRIGGER IF EXISTS update_blockchain_metrics_timestamp ON blockchain_metrics;
    CREATE TRIGGER update_blockchain_metrics_timestamp
    BEFORE UPDATE ON blockchain_metrics
    FOR EACH ROW
    EXECUTE FUNCTION update_timestamp();

    -- Create a function for upserting blockchain metrics
    CREATE OR REPLACE FUNCTION upsert_blockchain_metrics(
        p_block_height BIGINT,
        p_difficulty TEXT,
        p_connection_count INTEGER,
        p_tx_count INTEGER,
        p_block_size INTEGER,
        p_block_timestamp BIGINT,
        p_block_hash TEXT
    ) RETURNS VOID AS $$
    BEGIN
        INSERT INTO blockchain_metrics (
            block_height, difficulty, connection_count, tx_count, block_size, block_timestamp, block_hash
        ) VALUES (
            p_block_height, p_difficulty, p_connection_count, p_tx_count, p_block_size, p_block_timestamp, p_block_hash
        )
        ON CONFLICT (block_height) DO UPDATE SET
            difficulty = EXCLUDED.difficulty,
            connection_count = EXCLUDED.connection_count,
            tx_count = EXCLUDED.tx_count,
            block_size = EXCLUDED.block_size,
            block_timestamp = EXCLUDED.block_timestamp,
            block_hash = EXCLUDED.block_hash,
            timestamp = CURRENT_TIMESTAMP;
    END;
    $$ LANGUAGE plpgsql;
    ";

    // Execute all SQL statements
    client.batch_execute(CREATE_TABLE_SQL).await?;
    println!("Database initialized successfully");
    Ok(())
}
pub async fn setup_database(config: &DatabaseConfig) -> Result<Client, Box<dyn std::error::Error>> {
    let client = connect_to_postgres_with_retry(config).await?;
    initialize_database(&client).await?;
    Ok(client)
}
