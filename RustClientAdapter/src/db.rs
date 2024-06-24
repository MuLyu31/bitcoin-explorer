use tokio_postgres::{Client, NoTls};
use crate::config::DatabaseConfig;
use std::time::Duration;
use tokio::time::sleep;

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

pub async fn connect_to_postgres_with_retry(config: &DatabaseConfig) -> Result<Client, tokio_postgres::Error> {
    let mut retries = 5;
    let mut delay = Duration::from_secs(1);

    loop {
        match connect_to_postgres(config).await {
            Ok(client) => return Ok(client),
            Err(e) => {
                if retries == 0 {
                    eprintln!("Failed to connect to database after all retries. Last error: {}", e);
                    return Err(e);
                }
                eprintln!("Failed to connect to database: {}. Retrying in {:?}...", e, delay);
                sleep(delay).await;
                retries -= 1;
                delay *= 2;  // Exponential backoff
            }
        }
    }
}
