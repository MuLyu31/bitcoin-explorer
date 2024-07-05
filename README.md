# Bitcoin Exloperer
Intro: 

Team Member:

## 1. First Time Set-up
### 1.1 Back-End First Time Set Up
#### 1.1.1 Set up DB
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
    ```
    ```
    \c bitcoin_explorer
    ```
    ```
    CREATE TABLE transactions (
        id SERIAL PRIMARY KEY,
        txid VARCHAR(64) UNIQUE,
        block_height INT,
        fee BIGINT,
        -- Add more fields as needed
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );
    ```

- Check created schema by `\d`, should look like this:
    ```
    Schema |        Name         |   Type   |  Owner   
    --------+---------------------+----------+----------
    public | transactions        | table    | postgres
    public | transactions_id_seq | sequence | postgres
    (2 rows)
    ```
### 1.1.2. Install dependecy for rust
1. Install dep by `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. check ver, by `rustc --version`

### 1.1.3. Set up bitcoin client
- Install `brew install bitcoin`
- go to and set the user info conf (create this file if not existed):
`nano ~/Library/Application\ Support/Bitcoin/bitcoin.conf`

<!-- > You have to run the `bitcoin` first to get the folder above created

`brew services start bitcoin` (suppose you use MacOS and homebrew) -->

- type below into bitcoin.conf file
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

-------------------
### 1.2 Metric UI First Time Set Up

#### To Run the Grafana (For the second part of the project)
To visualize the data, you need to setup the Grafana host locally.
First time setup:
1. Install Grafana
`brew install grafana`


2. Start Grafana service
`brew services start grafana`

3. Open your browser and go to http://localhost:3000. The default login is admin/admin.

4.  Add PostgreSQL as a Data Source:
    Go to Configuration > Data Sources.
    Click Add data source and select PostgreSQL.
    Fill in the connection details:
    Host: localhost:5432
    Database: bitcoin_explorer
    User: postgres
    Password: 1234
    SSL Mode: Disable (if your PostgreSQL setup does not use SSL)
    Click Save & Test to verify the connection.

5. Create Dashboards with two panel, run below two queies for each panel
    - Panel for Block Height:
    ```
    SELECT created_at AS time, block_height
    FROM block_heights
    ORDER BY created_at;
    ```

    - Panel for Transaction Fees:
    ```
    SELECT created_at AS time, fee
    FROM transactions
    ORDER BY created_at;
    ```

## 2. To Run the Service
Preq: Finish section 1.1 first
### 2.1 To Run the RustClientAdapter/Back-end
- At root directory, `cd RustClientAdapter`
- `bitcoind -daemon`
- `brew services start postgresql@15`
- `psql -U postgres`
- `\c`
- Start a new terminal
- `cargo run`


### 2.2 To View the metrics visulization
- `brew services start grafana`
- go to  http://localhost:3000/login
- Login with credentials (username: Admin, password is ABC123 or something you choosed)

---
Add new line here
