Readme:

## To Run the RustClientAdapter
You need to 
1. Set up DB 
2. Install dependecy for rust
3. Set up bitcoin client

### 1. Set up DB
- Install
`
brew update
brew install postgresql
`
- start the service 
`brew services start postgresql@15`
- Creat user, and
`createuser --superuser postgres`
- login with user
`psql -U postgres`

- Set up schema
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

Created schema by `\d`, should look like this:
```
 Schema |        Name         |   Type   |  Owner   
--------+---------------------+----------+----------
 public | transactions        | table    | postgres
 public | transactions_id_seq | sequence | postgres
(2 rows)
```

### 2. Install dependecy for rust
1. Install dep by `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. check ver, by `rustc --version`

### 3. Set up bitcoin client
- Install `brew install bitcoin`
- go to and set the user info conf :

`nano ~/Library/Application\ Support/Bitcoin/bitcoin.conf`

> You have to run the `bitcoin` first to get the folder above created

`brew services start bitcoin`

type below into
````
server=1
rpcuser=myrpcuser
rpcpassword=myrpcpassword
rpcallowip=127.0.0.1
rpcport=8332
````
- Start the deamon by `bitcoind -daemon`
- To stop, run
`bitcoin-cli stop`
or
`bitcoin-cli -rpcuser=myrpcuser -rpcpassword=myrpcpassword stop`
