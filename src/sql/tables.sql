DROP TABLE accounts;
CREATE TABLE accounts (
    id INT4 PRIMARY KEY,
    balance BIGINT,
    created_at TIMESTAMP DEFAULT now(),
    creation_request BIGINT,
    CONSTRAINT balance_check CHECK (balance >= 0)
);
GRANT ALL ON TABLE accounts TO accountant;

DROP TABLE transactions;
CREATE TABLE transactions (
    id INT PRIMARY KEY DEFAULT unique_rowid(),
    transaction_index INT4,
    req_id BIGINT,
    account_id INT4,
    amount INT,
    created_at TIMESTAMP DEFAULT now()
);
GRANT ALL ON TABLE Transactions TO accountant;
