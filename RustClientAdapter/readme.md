Readme:

### Set up DB

brew update
brew install postgresql

brew services start postgresql@15
createuser --superuser postgres
psql -U postgres
```
CREATE DATABASE bitcoin_explorer;

\c bitcoin_explorer

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    txid VARCHAR(64) UNIQUE,
    block_height INT,
    fee BIGINT,
    -- Add more fields as needed
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

Created schema by `\d`
```
 Schema |        Name         |   Type   |  Owner   
--------+---------------------+----------+----------
 public | transactions        | table    | postgres
 public | transactions_id_seq | sequence | postgres
(2 rows)
```

### Set up rust service
1. Install dep by `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. check ver, by `rustc --version`
`