-- Create the blockchain_metrics table
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

-- Add a unique constraint on block_height
ALTER TABLE blockchain_metrics ADD CONSTRAINT unique_block_height UNIQUE (block_height);

-- Create indexes for faster queries
CREATE INDEX IF NOT EXISTS idx_blockchain_metrics_timestamp ON blockchain_metrics (timestamp);
CREATE INDEX IF NOT EXISTS idx_blockchain_metrics_block_height ON blockchain_metrics (block_height);

-- Create a function to update the timestamp
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.timestamp = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger to automatically update the timestamp
CREATE TRIGGER update_blockchain_metrics_timestamp
BEFORE UPDATE ON blockchain_metrics
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();

-- Create a sample record (optional, remove if not needed)
INSERT INTO blockchain_metrics (block_height, difficulty, connection_count, tx_count, block_size, block_timestamp, block_hash)
VALUES (0, '1', 0, 0, 0, 0, 'Genesis Block Hash')
ON CONFLICT (block_height) DO NOTHING;

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
--If init in this, we can use a clean way to insert or retrieve / update data in the table
