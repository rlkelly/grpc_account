extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

use postgres::error::T_R_SERIALIZATION_FAILURE;
use postgres::transaction::Transaction;
use postgres::{Connection, Error};
use postgres::transaction::{Config, IsolationLevel};
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use crate::proto::accounting::TransferComponent;
use crate::DataStore;

pub type PostgresPool = Pool<PostgresConnectionManager>;
pub type PostgresConnection = PooledConnection<PostgresConnectionManager>;
pub type PostgresResult<T> = Result<T, ()>;

// This will work with both Postgres and CockroachDb
#[derive(Clone)]
pub struct PostgresDataStore {
    pool: PostgresPool,
}

impl PostgresDataStore {
    pub fn new(conn_string: &str) -> PostgresDataStore {
        let manager = PostgresConnectionManager::new(conn_string, TlsMode::None).unwrap();
        let pool = Pool::new(manager).unwrap();
        PostgresDataStore { pool }
    }
    fn get_conn(&mut self) -> PostgresConnection {
        self.pool.get().unwrap()
    }
}

impl DataStore for PostgresDataStore {
    fn create_account(&mut self, account: u32, req_id: u64) -> PostgresResult<u64> {
        let res = create_account(self.get_conn(), account, req_id);
        match res {
            Ok(val) => Ok(val),
            _ => Err(()),
        }
    }

    fn get_account_balance(&mut self, account: i64) -> PostgresResult<i64> {
        let res = get_account_balance(self.get_conn(), account);
        match res {
            Ok(val) => Ok(val),
            _ => Err(()),
        }
    }

    fn execute_transfers(
        &mut self,
        transfers: &[TransferComponent],
        req_id: i64,
    ) -> PostgresResult<()> {
        let res = execute_transfers(self.get_conn(), transfers, req_id);
        match res {
            Ok(_) => Ok(()),
            _ => Err(()),
        }
    }
}

fn execute_txn<T, F>(conn: &Connection, op: F) -> Result<T, Error>
where
    F: Fn(&Transaction) -> Result<T, Error>,
{
    // Use serializable isolation to protect against concurrent writes
    let txn = conn.transaction()?;
    let mut cfg = Config::new();
    cfg.isolation_level(IsolationLevel::Serializable);
    txn.set_config(&cfg).unwrap();

    loop {
        let sp = txn.savepoint("cockroach_restart")?;
        match op(&sp).and_then(|t| sp.commit().map(|_| t)) {
            Err(ref err)
                if err
                    .as_db()
                    .map(|e| e.code == T_R_SERIALIZATION_FAILURE)
                    .unwrap_or(false) => {}
            r => break r,
        }
    }
    .and_then(|t| txn.commit().map(|_| t))
}

fn create_account(conn: PostgresConnection, account: u32, req_id: u64) -> Result<u64, Error> {
    conn.execute(
        "INSERT INTO accounts (balance, id, creation_request) VALUES ($1, $2, $3)",
        &[&100i64, &(account as i64), &(req_id as i64)],
    )
}

fn get_account_balance(conn: PostgresConnection, account: i64) -> Result<i64, Error> {
    let balance = conn.query("SELECT balance FROM accounts WHERE id=$1", &[&account])?;
    // If no rows are returned, need to inform user
    if balance.len() != 1 {
        Ok(-1)
    } else {
        Ok(balance.get(0).get(0))
    }
}

fn execute_transfers(
    conn: PostgresConnection,
    transfers: &[TransferComponent],
    req_id: i64,
) -> Result<(), Error> {
    execute_txn(&conn, |txn| transfer_funds(txn, transfers, req_id))
}

fn transfer_funds(
    txn: &Transaction,
    transfers: &[TransferComponent],
    req_id: i64,
) -> Result<(), Error> {
    for transfer in transfers {
        let delta: i64 = transfer.get_money_delta();
        let account = transfer.get_account_id() as i64;
        txn.execute(
            "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
            &[&delta, &account],
        )?;
        txn.execute(
            "INSERT INTO transactions (req_id, account_id, amount) VALUES ($1, $2, $3)",
            &[&req_id, &account, &delta],
        )?;
    }
    Ok(())
}

// fn ssl_config() -> OpenSsl {
//     // Warning! This API will be changing in the next version of these crates.
//     let mut connector_builder = SslConnectorBuilder::new(SslMethod::tls()).unwrap();
//     connector_builder.set_ca_file("certs/ca.crt").unwrap();
//     connector_builder.set_certificate_chain_file("certs/client.accountant.crt").unwrap();
//     connector_builder.set_private_key_file("certs/client.accountant.key", X509_FILETYPE_PEM).unwrap();
//
//     let mut ssl = OpenSsl::new().unwrap();
//     *ssl.connector_mut() = connector_builder.build();
//     ssl
// }
