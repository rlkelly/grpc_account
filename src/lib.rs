pub mod db;
pub mod proto;

use futures::Future;
use grpcio::{RpcContext, RpcStatus, RpcStatusCode, UnarySink};
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use crate::db::{
    create_account, execute_transfers, get_account_balance,
};
use crate::proto::accounting::{
    CreateAccountRequest, CreateAccountResponse, GetBalanceRequest, GetBalanceResponse,
    TransferRequest, TransferResponse,
};
use crate::proto::accounting_grpc::AccountingService;

pub type PostgresPool = Pool<PostgresConnectionManager>;
pub type PostgresConnection = PooledConnection<PostgresConnectionManager>;

#[derive(Clone)]
pub struct GrpcAccountingService {
    pool: PostgresPool,
}

impl GrpcAccountingService {
    pub fn new(conn_string: &str) -> GrpcAccountingService {
        let manager = PostgresConnectionManager::new(conn_string, TlsMode::None).unwrap();
        let pool = Pool::new(manager).unwrap();

        GrpcAccountingService { pool }
    }

    pub fn get_conn(&mut self) -> PostgresConnection {
        self.pool.get().unwrap()
    }
}

impl AccountingService for GrpcAccountingService {
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

        match create_account(self.get_conn(), account_id, req_id) {
            Ok(_) => {
                let f = sink
                    .success(reply)
                    .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                ctx.spawn(f);
            }
            Err(_) => {
                let f = sink
                    .fail(RpcStatus::new(
                        RpcStatusCode::InvalidArgument,
                        Some(String::from("Account Already Exists")),
                    ))
                    .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                ctx.spawn(f);
            }
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

        match get_account_balance(self.get_conn(), account_id as i64) {
            Ok(-1) => {
                let f = sink
                    .fail(RpcStatus::new(
                        RpcStatusCode::InvalidArgument,
                        Some(String::from("Failed To Find Account")),
                    ))
                    .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                ctx.spawn(f);
            }
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
            Err(_) => {
                let f = sink
                    .fail(RpcStatus::new(
                        RpcStatusCode::InvalidArgument,
                        Some(String::from("Server Error")),
                    ))
                    .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                ctx.spawn(f);
            }
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
            let f = sink
                .fail(RpcStatus::new(
                    RpcStatusCode::InvalidArgument,
                    Some(String::from("Sum of All Money Deltas Must Be Zero")),
                ))
                .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
            ctx.spawn(f);
        } else {
            match execute_transfers(self.get_conn(), &components, req_id as i64) {
                Ok(_) => {
                    let mut reply = TransferResponse::new();
                    reply.set_req_id(req_id);
                    let f = sink
                        .success(reply)
                        .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                    ctx.spawn(f);
                }
                Err(_) => {
                    let f = sink
                        .fail(RpcStatus::new(
                            RpcStatusCode::InvalidArgument,
                            Some(String::from("At Least One Account Had Invalid Balance")),
                        ))
                        .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                    ctx.spawn(f);
                }
            }
        }
    }
}
