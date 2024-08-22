CREATE TYPE BankAccountType AS ENUM ('CHECKING', 'INVESTMENT', 'CASH');

CREATE TABLE bank_accounts (
  id UUID,
  FOREIGN KEY (user_id) REFERENCES users(id),
  name VARCHAR(255) NOT NULL,
  type BankAccountType NOT NULL,
  balance INTEGER NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT "bank_accounts_pkey" PRIMARY KEY ("id"),
);

CREATE UNIQUE INDEX "bank_accounts_id_key" ON "bank_accounts"("id");
