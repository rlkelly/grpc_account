### Accounting GRPC Server

# Tooling:
  ## Rust Libraries
  1) postgres: rust crate for postgres
  2) grpcio: rust crate for grpc
  3) protobuf: rust crate for serialization
  4) futures: frust crate for creating promises
  5) r2d2, r2d2_postgres: rust crates for Connection Pools for Postgres
  6) threadpool: async testing

  ## Database
  - Postgres or CockroachDb.  I chose CockroachDb because of its distribution and high availability, as well as its default isolation for transactions is Serialization, which fits this problem nicely.

## QUICKSTART:
```bash
$ brew install cockroachdb
$ cockroach start --insecure --listen-addr=localhost
$ cockroach sql --insecure < sql/database.sql
$ cargo run server
```

## Tests
   Be sure to run single threaded as they rely on the same tables.
``` bash
$ RUST_TEST_THREADS=1 cargo test
```

## Todo
   - Improve Multithreaded tests.
   - Benchmarking
