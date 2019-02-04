pub mod db;
pub mod proto;

use futures::Future;
use grpcio::{RpcContext, RpcStatus, RpcStatusCode, UnarySink};

use crate::proto::accounting::TransferComponent;
use crate::proto::accounting::{
    CreateAccountRequest, CreateAccountResponse, GetBalanceRequest, GetBalanceResponse,
    TransferRequest, TransferResponse,
};
use crate::proto::accounting_grpc::AccountingService;

pub trait DataStore {
    fn create_account(&mut self, account: u32, req_id: u64) -> Result<u64, ()>;
    fn get_account_balance(&mut self, account: i64) -> Result<i64, ()>;
    fn execute_transfers(&mut self, transfers: &[TransferComponent], req_id: i64)
        -> Result<(), ()>;
}

#[derive(Clone)]
pub struct GrpcAccountingService<T>
where
    T: 'static + DataStore + Send + Sync,
{
    store: T,
}

impl<T> GrpcAccountingService<T>
where
    T: 'static + DataStore + Send + Sync,
{
    pub fn new(store: T) -> GrpcAccountingService<T> {
        GrpcAccountingService { store }
    }

    fn send_error<U>(
        &self,
        sink: UnarySink<U>,
        ctx: RpcContext,
        status_code: RpcStatusCode,
        arg_string: &str,
    ) {
        let f = sink
            .fail(RpcStatus::new(status_code, Some(arg_string.to_string())))
            .map_err(move |e| println!("failed to reply: {:?}", e));
        ctx.spawn(f);
    }
}

impl<T> AccountingService for GrpcAccountingService<T>
where
    T: 'static + DataStore + Send + Sync,
{
    fn create_account(
        &mut self,
        ctx: RpcContext,
        req: CreateAccountRequest,
        sink: UnarySink<CreateAccountResponse>,
    ) {
        let req_id = req.get_req_id();
        let account_id = req.get_account_id();
        let mut reply = CreateAccountResponse::new();
        reply.set_req_id(req_id);
        reply.set_account_id(account_id);

        match self.store.create_account(account_id, req_id) {
            Ok(_) => {
                let f = sink
                    .success(reply)
                    .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                ctx.spawn(f);
            }
            Err(_) => self.send_error(
                sink,
                ctx,
                RpcStatusCode::InvalidArgument,
                "Account Already Exists",
            ),
        }
    }

    fn get_balance(
        &mut self,
        ctx: RpcContext,
        req: GetBalanceRequest,
        sink: UnarySink<GetBalanceResponse>,
    ) {
        let req_id = req.get_req_id();
        let account_id = req.get_account_id();

        match self.store.get_account_balance(account_id as i64) {
            Ok(-1) => self.send_error(
                sink,
                ctx,
                RpcStatusCode::NotFound,
                "Resource Not Found",
            ),
            Ok(balance) => {
                let mut reply = GetBalanceResponse::new();
                reply.set_req_id(req_id);
                reply.set_account_id(account_id);
                reply.set_balance(balance);
                let f = sink
                    .success(reply)
                    .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                ctx.spawn(f);
            }
            Err(_) => self.send_error(sink, ctx, RpcStatusCode::Unknown, "Server Error"),
        };
    }

    fn transfer(
        &mut self,
        ctx: RpcContext,
        req: TransferRequest,
        sink: UnarySink<TransferResponse>,
    ) {
        let req_id = req.get_req_id();
        let components = req.get_components();
        let verify_total = components
            .into_iter()
            .fold(0, |sum, i| sum + i.get_money_delta());

        if verify_total != 0 {
            self.send_error(
                sink,
                ctx,
                RpcStatusCode::FailedPrecondition,
                "Sum of All Money Deltas Must Be Zero",
            );
        } else {
            match self.store.execute_transfers(&components, req_id as i64) {
                Ok(_) => {
                    let mut reply = TransferResponse::new();
                    reply.set_req_id(req_id);
                    let f = sink
                        .success(reply)
                        .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                    ctx.spawn(f);
                }
                Err(_) => self.send_error(
                    sink,
                    ctx,
                    RpcStatusCode::Aborted,
                    "Transaction Error.  Possibly a User Had Insufficient Funds",
                ),
            }
        }
    }
}
