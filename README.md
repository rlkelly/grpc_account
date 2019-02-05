### Accounting GRPC Server

# Tooling:
  ## Rust Libraries
<<<<<<< HEAD
  - 1) postgres: rust crate for postgres
  - 2) grpcio: rust crate for grpc
  - 3) protobuf: rust crate for serialization
  - 4) futures: frust crate for creating promises
  - 5) r2d2, r2d2_postgres: rust crates for Connection Pools for Postgres
  - 6) threadpool: async testing

  ## Database
  - 1) Postgres or CockroachDb.  I chose CockroachDb because of its distribution
       and high availability, as well as its default isolation for transactions is Serialization, which fits this problem nicely.

QUICKSTART:
    1) Install Postgres or CockroachDb
    2) Run in insecure mode on port 26257
      a) Create database bank
      b) Connection string should be: "postgresql://accountant@localhost:26257/bank"
    3) run src/sql/database.sql
    4) run src/sql/tables.sql
    5) cargo run server
=======
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
>>>>>>> 79f168d3e32dd07cde166b368c49eb934196fe17
