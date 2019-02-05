use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Instant;
use threadpool::ThreadPool;

use grpcio::{ChannelBuilder, EnvBuilder};

use accountant::proto::accounting::{
    CreateAccountRequest, GetBalanceRequest, TransferComponent, TransferRequest,
    ResetRequest,
};
use accountant::proto::accounting_grpc::AccountingServiceClient;

#[derive(Clone)]
pub struct ReqCounter {
    req_id: Arc<AtomicUsize>,
}

impl ReqCounter {
    pub fn incr(&self) -> u64 {
        (self.req_id.fetch_add(1, Ordering::Relaxed)).clone() as u64
    }
}

pub fn make_create_request(req_id: u64, account_id: u32, balance: i64) -> CreateAccountRequest {
    let mut req = CreateAccountRequest::new();
    req.set_req_id(req_id);
    req.set_account_id(account_id);
    req.set_balance(balance);
    req
}

pub fn make_balance_request(req_id: u64, account_id: u32) -> GetBalanceRequest {
    let mut req = GetBalanceRequest::new();
    req.set_req_id(req_id);
    req.set_account_id(account_id);
    req
}

pub fn make_transfer_request(req_id: u64, transactions: &[(u32, i64)]) -> TransferRequest {
    let mut req = TransferRequest::new();
    req.set_req_id(req_id);

    for transaction in transactions {
        let (account_id, amount) = transaction;
        let mut comp = TransferComponent::new();
        comp.set_account_id(*account_id);
        comp.set_money_delta(*amount);
        req.mut_components().push(comp);
    }
    req
}

pub fn create_transfer_list(counter: &ReqCounter) -> Vec<TransferRequest> {
    let mut transfers: Vec<TransferRequest> = Vec::new();
    for _ in 0..1_000 {
        let req = make_transfer_request(counter.incr(), &[(10_000, -4), (102, 2), (103, 2)]);
        transfers.push(req);
        let req = make_transfer_request(counter.incr(), &[(10_000, 2), (102, -1), (103, -1)]);
        transfers.push(req);
    }
    transfers
}

fn main() {
    let start = Instant::now();
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("localhost:3000");
    let client = AccountingServiceClient::new(ch);
    let counter = ReqCounter {
        req_id: Arc::new(AtomicUsize::new(0)),
    };

    // reset database
    let req = ResetRequest::new();
    client.reset(&req).expect("Database Reset Failed");

    // make our large donor
    let req = make_create_request(counter.incr(), 10_000, 10_000);
    let reply = client.create_account(&req);
    assert!(reply.is_ok());

    let pool = ThreadPool::new(20);
    let barrier = Arc::new(Barrier::new(20));
    for i in 1..20 {
        let barrier = barrier.clone();
        let counter = counter.clone();
        let client = client.clone();
        pool.execute(move || {
            for j in (100 * i)..(100 * (i + 1)) {
                let req = make_create_request(counter.incr(), j, 1_000);
                let reply = client.create_account(&req);
                assert!(reply.is_ok());
            }
            barrier.wait();
        });
    }
    barrier.wait();

    // redundant create request
    let req = make_create_request(counter.incr(), 129, 1_000);
    let reply = client.create_account(&req);
    assert!(reply.is_err());

    // get balance of valid account
    let req = make_balance_request(counter.incr(), 129);
    let reply = client.get_balance(&req);
    assert!(reply.is_ok());

    // get balance of invalid account
    let req = make_balance_request(counter.incr(), 55_000);
    let reply = client.get_balance(&req);
    assert!(reply.is_err());

    // test transfer with non existant accounts
    let req = make_transfer_request(counter.incr(), &[(10, -50), (105, 50)]);
    let reply = client.transfer(&req);
    assert!(reply.is_err());

    let req = make_transfer_request(counter.incr(), &[(105, -50), (10, 50)]);
    let reply = client.transfer(&req);
    assert!(reply.is_err());

    // test transfer with invalid sum
    let req = make_transfer_request(counter.incr(), &[(10, -50), (11, 55)]);
    let reply = client.transfer(&req);
    assert!(reply.is_err());

    // test transfer with negative balance
    let req = make_transfer_request(counter.incr(), &[(10, -2_000), (11, 2_000)]);
    let reply = client.transfer(&req);
    assert!(reply.is_err());

    // test for valid transfer
    let req = make_transfer_request(counter.incr(), &[(101, -50), (102, 25), (103, 25)]);
    let reply = client.transfer(&req);
    assert!(reply.is_ok());

    let req = make_transfer_request(counter.incr(), &[(200, -50), (200, -50), (300, 100)]);
    let reply = client.transfer(&req);
    assert!(reply.is_ok());

    let transfers = create_transfer_list(&counter);
    let pool = ThreadPool::new(2_000);
    let barrier = Arc::new(Barrier::new(2_000));
    for transfer in transfers {
        let barrier = barrier.clone();
        let client = client.clone();
        pool.execute(move || {
            let reply = client.transfer(&transfer);
            assert!(reply.is_ok());
            barrier.wait();
        });
    }
    barrier.wait();

    // TODO: check balance against expected
    let req = make_balance_request(counter.incr(), 10_000);
    let reply = client.get_balance(&req).expect("failed to get balance for large donor");
    assert_eq!(reply.get_balance(), 8000);

    println!("{:?}", counter.req_id);
    let elapsed = start.elapsed();
    println!(
        "Elapsed: {} ms",
        (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64
    );
}
