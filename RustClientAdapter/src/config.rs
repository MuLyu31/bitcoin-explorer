use std::env;

pub struct Config {
    pub use_api: bool,
    pub db_config: DatabaseConfig,
}

pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        let use_api = env::var("USE_API").unwrap_or_else(|_| "false".to_string()) == "true";
        let db_config = DatabaseConfig::from_env();

        Config { use_api, db_config }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        DatabaseConfig {
            host: env::var("DB_HOST").expect("DB_HOST must be set"),
            port: env::var("DB_PORT")
                .expect("DB_PORT must be set")
                .parse()
                .expect("DB_PORT must be a valid port number"),
            username: env::var("DB_USERNAME").expect("DB_USER must be set"),
            password: env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),
            database_name: env::var("DB_NAME").expect("DB_NAME must be set"),
        }
    }
}
