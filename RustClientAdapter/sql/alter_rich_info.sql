ALTER TABLE blockchain_metrics
ADD COLUMN tx_count INTEGER,
ADD COLUMN block_size INTEGER,
ADD COLUMN block_timestamp BIGINT,
ADD COLUMN block_hash TEXT;

-- Create an index on block_height for faster queries
CREATE INDEX IF NOT EXISTS idx_blockchain_metrics_block_height ON blockchain_metrics (block_height);
-- Add a unique constraint on block_height
ALTER TABLE blockchain_metrics ADD CONSTRAINT unique_block_height UNIQUE (block_height);

-- If you want to allow updates to existing rows, you might want to use this instead:
-- CREATE UNIQUE INDEX unique_block_height ON blockchain_metrics (block_height);
