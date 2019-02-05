use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use threadpool::ThreadPool;

use grpcio::{ChannelBuilder, EnvBuilder, Result};

use accountant::proto::accounting::{
    CreateAccountRequest, GetBalanceRequest, TransferComponent, TransferRequest,
    ResetRequest, ResetResponse, CreateAccountResponse, TransferResponse,
    GetBalanceResponse,
};
use accountant::proto::accounting_grpc::AccountingServiceClient;

#[derive(Clone)]
pub struct ReqCounter {
    req_id: Arc<AtomicUsize>,
}

impl ReqCounter {
    pub fn new() -> ReqCounter {
        ReqCounter {
            req_id: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn incr(&self) -> u64 {
        (self.req_id.fetch_add(1, Ordering::Relaxed)).clone() as u64
    }
}

#[derive(Clone)]
struct AccountTestingClient {
    client: AccountingServiceClient,
    counter: ReqCounter,
}

impl AccountTestingClient {
    pub fn new() -> AccountTestingClient {
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect("localhost:3000");
        let client = AccountingServiceClient::new(ch);
        let counter = ReqCounter::new();
        AccountTestingClient {
            client,
            counter,
        }
    }

    pub fn reset(&self) -> Result<ResetResponse> {
        let req = ResetRequest::new();
        self.client.reset(&req)
    }

    pub fn create_account(&mut self, account: u32, balance: i64) -> Result<CreateAccountResponse> {
        let mut req = CreateAccountRequest::new();
        req.set_req_id(self.counter.incr());
        req.set_account_id(account);
        req.set_balance(balance);
        self.client.create_account(&req)
    }

    pub fn get_balance(&mut self, account: u32) -> Result<GetBalanceResponse> {
        let mut req = GetBalanceRequest::new();
        req.set_req_id(self.counter.incr());
        req.set_account_id(account);
        self.client.get_balance(&req)
    }

    pub fn transfer(&self, transactions: &[(u32, i64)]) -> Result<TransferResponse> {
        let mut req = TransferRequest::new();
        req.set_req_id(self.counter.incr());

        for transaction in transactions {
            let (account_id, amount) = transaction;
            let mut comp = TransferComponent::new();
            comp.set_account_id(*account_id);
            comp.set_money_delta(*amount);
            req.mut_components().push(comp);
        }
        self.client.transfer(&req)
    }

    pub fn create_test_transfers(&mut self) -> Vec<Vec<(u32, i64)>> {
        let mut transfers: Vec<Vec<(u32, i64)>> = Vec::new();
        for _ in 0..500 {
            let req = [(1, -4), (2, 2), (3, 2)];
            transfers.push(req.to_vec());
            let req = &[(1, 2), (2, -1), (3, -1)];
            transfers.push(req.to_vec());
        }
        transfers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account() {
        let mut client = AccountTestingClient::new();
        client.reset().expect("Database Reset Failed");

        let pool = ThreadPool::new(20);
        let barrier = Arc::new(Barrier::new(20));
        for i in 1..20 {
            let barrier = barrier.clone();
            let mut client = client.clone();
            pool.execute(move || {
                for account_id in (100 * i)..(100 * (i + 1)) {
                    let reply = client.create_account(account_id, 1_000);
                    assert!(reply.is_ok());
                }
                barrier.wait();
            });
        }
        barrier.wait();

        for account in 100..2000 {
            let reply = client.get_balance(account);
            assert!(reply.is_ok());
            assert_eq!(reply.unwrap().get_balance(), 1_000);
        }

        // redundant create request
        let reply = client.create_account(100, 1_000);
        assert!(reply.is_err());

    }

    #[test]
    fn test_get_balance() {
        let mut client = AccountTestingClient::new();
        client.reset().expect("Database Reset Failed");

        let reply = client.create_account(1, 1_000);
        assert!(reply.is_ok());

        // get balance of valid account
        let reply = client.get_balance(1);
        assert!(reply.is_ok());
        assert_eq!(reply.unwrap().get_balance(), 1_000);

        // get balance of invalid account
        let reply = client.get_balance(2);
        assert!(reply.is_err());
    }

    #[test]
    fn test_transfer() {
        let mut client = AccountTestingClient::new();
        client.reset().expect("Database Reset Failed");

        // make our large donor
        let reply = client.create_account(1, 5_000);
        assert!(reply.is_ok());
        let reply = client.create_account(2, 0);
        assert!(reply.is_ok());
        let reply = client.create_account(3, 0);
        assert!(reply.is_ok());

        // test transfer with non existant accounts
        let reply = client.transfer(&[(1, -50), (5, 50)]);
        assert!(reply.is_err());

        let reply = client.transfer(&[(5, -50), (1, 50)]);
        assert!(reply.is_err());

        let reply = client.transfer(&[(5, -50), (6, 50)]);
        assert!(reply.is_err());

        // test transfer with invalid sum
        let reply = client.transfer(&[(1, -50), (2, 55)]);
        assert!(reply.is_err());

        // test transfer with negative balance
        let reply = client.transfer(&[(1, -10_000), (2, 10_000)]);
        assert!(reply.is_err());

        // test for valid transfer
        let reply = client.transfer(&[(1, -50), (2, 25), (3, 25)]);
        assert!(reply.is_ok());

        let reply = client.transfer(&[(1, 50), (2, -25), (3, -25)]);
        assert!(reply.is_ok());

        let transfers = client.create_test_transfers();
        let pool = ThreadPool::new(4);
        for transfer in transfers {
            let client = client.clone();
            pool.execute(move || {
                let reply = client.transfer(&transfer);
                assert!(reply.is_ok());
            });
        }
        pool.join();

        let reply = client.get_balance(1).expect("failed to get balance for large donor");
        assert_eq!(reply.get_balance(), 4_000);

        let reply = client.get_balance(2).expect("failed to get balance for large donor");
        assert_eq!(reply.get_balance(), 500);

        let reply = client.get_balance(3).expect("failed to get balance for large donor");
        assert_eq!(reply.get_balance(), 500);
    }
}
