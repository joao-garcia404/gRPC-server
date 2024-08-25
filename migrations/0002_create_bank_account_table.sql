DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'bankaccounttype') THEN
        CREATE TYPE BankAccountType AS ENUM ('CHECKING', 'INVESTMENT', 'CASH');
    END IF;
END $$;

CREATE TABLE bank_accounts (
  id UUID,
  user_id UUID,
  name VARCHAR(255) NOT NULL,
  type BankAccountType NOT NULL,
  balance INTEGER NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users(id),

  CONSTRAINT "bank_accounts_pkey" PRIMARY KEY ("id")
);

CREATE UNIQUE INDEX "bank_accounts_id_key" ON "bank_accounts"("id");
