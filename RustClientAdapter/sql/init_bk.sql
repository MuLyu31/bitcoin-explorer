CREATE TABLE blockchain_metrics (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    block_height INTEGER NOT NULL,
    difficulty TEXT NOT NULL,
    connection_count INTEGER
);

CREATE INDEX IF NOT EXISTS idx_blockchain_metrics_timestamp ON blockchain_metrics (timestamp);
