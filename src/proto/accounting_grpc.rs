// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_ACCOUNTING_SERVICE_CREATE_ACCOUNT: ::grpcio::Method<super::accounting::CreateAccountRequest, super::accounting::CreateAccountResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/accounting.AccountingService/CreateAccount",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_ACCOUNTING_SERVICE_GET_BALANCE: ::grpcio::Method<super::accounting::GetBalanceRequest, super::accounting::GetBalanceResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/accounting.AccountingService/GetBalance",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_ACCOUNTING_SERVICE_TRANSFER: ::grpcio::Method<super::accounting::TransferRequest, super::accounting::TransferResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/accounting.AccountingService/Transfer",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct AccountingServiceClient {
    client: ::grpcio::Client,
}

impl AccountingServiceClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        AccountingServiceClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn create_account_opt(&self, req: &super::accounting::CreateAccountRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::accounting::CreateAccountResponse> {
        self.client.unary_call(&METHOD_ACCOUNTING_SERVICE_CREATE_ACCOUNT, req, opt)
    }

    pub fn create_account(&self, req: &super::accounting::CreateAccountRequest) -> ::grpcio::Result<super::accounting::CreateAccountResponse> {
        self.create_account_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_account_async_opt(&self, req: &super::accounting::CreateAccountRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::accounting::CreateAccountResponse>> {
        self.client.unary_call_async(&METHOD_ACCOUNTING_SERVICE_CREATE_ACCOUNT, req, opt)
    }

    pub fn create_account_async(&self, req: &super::accounting::CreateAccountRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::accounting::CreateAccountResponse>> {
        self.create_account_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_balance_opt(&self, req: &super::accounting::GetBalanceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::accounting::GetBalanceResponse> {
        self.client.unary_call(&METHOD_ACCOUNTING_SERVICE_GET_BALANCE, req, opt)
    }

    pub fn get_balance(&self, req: &super::accounting::GetBalanceRequest) -> ::grpcio::Result<super::accounting::GetBalanceResponse> {
        self.get_balance_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_balance_async_opt(&self, req: &super::accounting::GetBalanceRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::accounting::GetBalanceResponse>> {
        self.client.unary_call_async(&METHOD_ACCOUNTING_SERVICE_GET_BALANCE, req, opt)
    }

    pub fn get_balance_async(&self, req: &super::accounting::GetBalanceRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::accounting::GetBalanceResponse>> {
        self.get_balance_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn transfer_opt(&self, req: &super::accounting::TransferRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::accounting::TransferResponse> {
        self.client.unary_call(&METHOD_ACCOUNTING_SERVICE_TRANSFER, req, opt)
    }

    pub fn transfer(&self, req: &super::accounting::TransferRequest) -> ::grpcio::Result<super::accounting::TransferResponse> {
        self.transfer_opt(req, ::grpcio::CallOption::default())
    }

    pub fn transfer_async_opt(&self, req: &super::accounting::TransferRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::accounting::TransferResponse>> {
        self.client.unary_call_async(&METHOD_ACCOUNTING_SERVICE_TRANSFER, req, opt)
    }

    pub fn transfer_async(&self, req: &super::accounting::TransferRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::accounting::TransferResponse>> {
        self.transfer_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait AccountingService {
    fn create_account(&mut self, ctx: ::grpcio::RpcContext, req: super::accounting::CreateAccountRequest, sink: ::grpcio::UnarySink<super::accounting::CreateAccountResponse>);
    fn get_balance(&mut self, ctx: ::grpcio::RpcContext, req: super::accounting::GetBalanceRequest, sink: ::grpcio::UnarySink<super::accounting::GetBalanceResponse>);
    fn transfer(&mut self, ctx: ::grpcio::RpcContext, req: super::accounting::TransferRequest, sink: ::grpcio::UnarySink<super::accounting::TransferResponse>);
}

pub fn create_accounting_service<S: AccountingService + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_ACCOUNTING_SERVICE_CREATE_ACCOUNT, move |ctx, req, resp| {
        instance.create_account(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_ACCOUNTING_SERVICE_GET_BALANCE, move |ctx, req, resp| {
        instance.get_balance(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_ACCOUNTING_SERVICE_TRANSFER, move |ctx, req, resp| {
        instance.transfer(ctx, req, resp)
    });
    builder.build()
}
