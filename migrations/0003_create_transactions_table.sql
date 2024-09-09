DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'transactiontype') THEN
        CREATE TYPE TransactionType AS ENUM ('INCOME', 'OUTCOME');
    END IF;
END $$;

CREATE TABLE transactions (
  id UUID,
  amount BIGINT NOT NULL,
  transaction_type TransactionType NOT NULL,
  origin_account_id UUID NOT NULL,
  description VARCHAR(255) DEFAULT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (origin_account_id) REFERENCES bank_accounts(id),

  CONSTRAINT "transactions_pkey" PRIMARY KEY ("id")
);