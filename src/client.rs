use std::sync::{Arc, Barrier};
use std::time::Instant;
use threadpool::ThreadPool;
use std::sync::atomic::{AtomicUsize, Ordering};

use grpcio::{ChannelBuilder, EnvBuilder};

use accountant::proto::accounting::{
    CreateAccountRequest, GetBalanceRequest, TransferComponent, TransferRequest,
};
use accountant::proto::accounting_grpc::AccountingServiceClient;

pub fn make_create_request(req_id: u64, account_id: u32) -> CreateAccountRequest {
    let mut req = CreateAccountRequest::new();
    req.set_req_id(req_id);
    req.set_account_id(account_id);
    req
}

pub fn make_balance_request(req_id: u64, account_id: u32) -> GetBalanceRequest {
    let mut req = GetBalanceRequest::new();
    req.set_req_id(req_id);
    req.set_account_id(account_id);
    req
}

fn main() {
    let start = Instant::now();
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("localhost:3000");
    let client = AccountingServiceClient::new(ch);

    let mut req_id = 0;

    // let req = make_create_request(18446744073709551600, 2005);
    // let reply = client.create_account(&req);
    // assert!(reply.is_ok());

    let pool = ThreadPool::new(20);
    let barrier = Arc::new(Barrier::new(20));
    let an_atomic = Arc::new(AtomicUsize::new(0));

    for i in 1..20 {
      let barrier = barrier.clone();
      let an_atomic = an_atomic.clone();
      pool.execute(move|| {
          let env = Arc::new(EnvBuilder::new().build());
          let ch = ChannelBuilder::new(env).connect("localhost:3000");
          let client = AccountingServiceClient::new(ch);
          an_atomic.fetch_add(1, Ordering::Relaxed);
          let req = make_create_request(req_id, i);
          // req_id += 1;
          let reply = client.create_account(&req);
          // assert!(reply.is_ok());
          barrier.wait();
      });
    }
    barrier.wait();

    let req = make_create_request(req_id, 10);
    req_id += 1;
    let reply = client.create_account(&req);
    assert!(reply.is_err());

    let req = make_balance_request(req_id, 15);
    req_id += 1;
    let reply = client.get_balance(&req);
    assert!(reply.is_ok());

    let req = make_balance_request(req_id, 5500);
    req_id += 1;
    let reply = client.get_balance(&req);
    assert!(reply.is_err());

    let mut req = TransferRequest::new();
    req.set_req_id(req_id);

    let mut comp = TransferComponent::new();
    comp.set_account_id(10);
    comp.set_money_delta(-55);
    req.mut_components().push(comp);
    let mut comp = TransferComponent::new();
    comp.set_account_id(10);
    comp.set_money_delta(-50);
    req.mut_components().push(comp);
    let mut comp = TransferComponent::new();
    comp.set_account_id(11);
    comp.set_money_delta(105);
    req.mut_components().push(comp);
    let reply = client.transfer(&req);
    assert!(reply.is_err());

    let mut req = TransferRequest::new();
    req.set_req_id(req_id);
    req_id += 1;
    let mut comp = TransferComponent::new();
    comp.set_account_id(10);
    comp.set_money_delta(-45);
    req.mut_components().push(comp);
    let mut comp = TransferComponent::new();
    comp.set_account_id(10);
    comp.set_money_delta(-50);
    req.mut_components().push(comp);
    let mut comp = TransferComponent::new();
    comp.set_account_id(11);
    comp.set_money_delta(105);
    req.mut_components().push(comp);

    let reply = client.transfer(&req);
    assert!(reply.is_err());

    let mut req = TransferRequest::new();
    req.set_req_id(req_id);
    let mut comp = TransferComponent::new();
    comp.set_account_id(11);
    comp.set_money_delta(-50);
    req.mut_components().push(comp);
    let mut comp = TransferComponent::new();
    comp.set_account_id(11);
    comp.set_money_delta(-50);
    req.mut_components().push(comp);
    let mut comp = TransferComponent::new();
    comp.set_account_id(12);
    comp.set_money_delta(100);
    req.mut_components().push(comp);
    let reply = client.transfer(&req);
    assert!(reply.is_ok());

    println!("{:?}", an_atomic);
    let elapsed = start.elapsed();
    println!("Elapsed: {} ms",
             (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64);
}
