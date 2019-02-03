extern crate postgres;

use postgres::error::T_R_SERIALIZATION_FAILURE;
use postgres::transaction::Transaction;
use postgres::{Connection, Result};

use crate::proto::accounting::TransferComponent;
use crate::PostgresConnection;

fn execute_txn<T, F>(conn: &Connection, op: F) -> Result<T>
where
    F: Fn(&Transaction) -> Result<T>,
{
    let txn = conn.transaction()?;
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

pub fn create_account(conn: PostgresConnection, account: u32, req_id: u64) -> Result<u64> {
    conn.execute(
        "INSERT INTO accounts (balance, id, creation_request) VALUES ($1, $2, $3)",
        &[&100i64, &(account as i64), &(req_id as i64)],
    )
}

pub fn get_account_balance(conn: PostgresConnection, account: i64) -> Result<i64> {
    let balance = conn.query("SELECT balance FROM accounts WHERE id=$1", &[&account])?;
    if balance.len() != 1 {
        Ok(-1)
    } else {
        Ok(balance.get(0).get(0))
    }
}

pub fn execute_transfers(
    conn: PostgresConnection,
    transfers: &[TransferComponent],
    req_id: i64,
) -> Result<()> {
    execute_txn(&conn, |txn| transfer_funds(txn, transfers, req_id))
}

fn transfer_funds(txn: &Transaction, transfers: &[TransferComponent], req_id: i64) -> Result<()> {
    // Perform the transfers.
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

// fn main() {
//     let tls_mode = TlsMode::Require(&ssl_config());
//     let conn = Connection::connect("postgresql://accountant@localhost:26257/bank", tls_mode)
//         .unwrap();
// }
