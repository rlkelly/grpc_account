use std::io::Read;
use std::sync::Arc;
use std::{io, thread};

use futures::sync::oneshot;
use futures::Future;
use grpcio::{Environment, ServerBuilder, ChannelBuilder};

use accountant::proto::accounting_grpc;
use accountant::GrpcAccountingService;
use accountant::db::PostgresDataStore;

fn main() {
    let env = Arc::new(Environment::new(4));

    let channel_args = ChannelBuilder::new(Arc::clone(&env))
        .stream_initial_window_size(2 * 1024 * 1024)
        .max_concurrent_stream(1024)
        .max_send_message_len(32 * 1024 * 1024)
        .max_receive_message_len(32 * 1024 * 1024)
        .build_args();

    let service = accounting_grpc::create_accounting_service(
        GrpcAccountingService::new(
            PostgresDataStore::new("postgresql://accountant@localhost:26257/bank"))
    );

    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("0.0.0.0", 3000)
        .channel_args(channel_args)
        .build()
        .unwrap();

    server.start();
    for &(ref host, port) in server.bind_addrs() {
        println!("Server listening on {}:{}", host, port);
    }

    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });

    let _ = rx.wait();
    let _ = server.shutdown().wait();
}
