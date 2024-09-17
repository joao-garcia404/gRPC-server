use std::fmt;

use chrono::Utc;
use sqlx::{
    postgres::{PgRow, Postgres},
    FromRow, Row,
};
use thiserror::Error;
use uuid::Uuid;

use crate::models::transaction::{Transaction, TransactionType};

#[derive(Error, Debug)]
enum BankAccountErrorType {
    #[error("Invalid account type")]
    InvalidAccountType,
    #[error("Failed to parse user id as a UUID")]
    UserIdParse,
    #[error("The account don't have enough funds to complete the transaction")]
    NotEnoughFunds,
}

#[derive(Debug, Error)]
pub struct BankAccountError {
    action: BankAccountErrorType,
}

impl BankAccountError {
    fn new(action: BankAccountErrorType) -> Self {
        BankAccountError { action }
    }

    pub fn account_type() -> Self {
        BankAccountError::new(BankAccountErrorType::InvalidAccountType)
    }

    pub fn user_id_parse() -> Self {
        BankAccountError::new(BankAccountErrorType::UserIdParse)
    }

    pub fn not_enough_funds() -> Self {
        BankAccountError::new(BankAccountErrorType::NotEnoughFunds)
    }
}

impl std::fmt::Display for BankAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.action)
    }
}

#[derive(sqlx::Type, Debug, Clone, PartialEq)]
#[sqlx(type_name = "bankaccounttype", rename_all = "UPPERCASE")]
pub enum AccountType {
    CHECKING,
    INVESTMENT,
    CASH,
}

impl AccountType {
    pub fn from_raw_string(raw: &str) -> Result<AccountType, BankAccountError> {
        match raw {
            "CHECKING" => Ok(AccountType::CHECKING),
            "INVESTMENT" => Ok(AccountType::INVESTMENT),
            "CASH" => Ok(AccountType::CASH),
            _ => Err(BankAccountError::account_type()),
        }
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountType::CASH => write!(f, "CASH"),
            AccountType::CHECKING => write!(f, "CHECKING"),
            AccountType::INVESTMENT => write!(f, "INVESTMENT"),
        }
    }
}

#[derive(Debug)]
pub struct BankAccount {
    pub id: Uuid,
    pub name: String,
    pub balance: f64,
    pub account_type: AccountType,
    pub user_id: Uuid,
    pub created_at: String,
}

impl BankAccount {
    pub fn new(
        name: String,
        balance: f64,
        account_type: AccountType,
        user_id: String,
    ) -> Result<BankAccount, BankAccountError> {
        let user_id =
            Uuid::try_parse(user_id.as_str()).map_err(|_err| BankAccountError::user_id_parse())?;

        Ok(BankAccount {
            id: Uuid::new_v4(),
            name,
            balance,
            account_type,
            user_id,
            created_at: Utc::now().to_rfc3339(),
        })
    }

    pub fn update_balance(&mut self, transaction: &Transaction) -> Result<(), BankAccountError> {
        match transaction.transaction_type {
            TransactionType::OUTCOME => {
                if self.balance < transaction.amount {
                    return Err(BankAccountError::not_enough_funds());
                }

                self.balance -= transaction.amount;

                Ok(())
            }
            TransactionType::INCOME => {
                self.balance += transaction.amount;
                Ok(())
            }
        }
    }

    pub fn from_pg_row(row: PgRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = row.get("id");
        let name: String = row.get("name");
        let balance_in_cents: i64 = row.try_get("balance")?;
        let balance: f64 = balance_in_cents as f64 / 100.0;
        let account_type: AccountType = row.get("type");
        let user_id: Uuid = row.get("user_id");
        let created_at: String = row.get("created_at");

        Ok(BankAccount {
            id,
            name,
            balance,
            account_type,
            user_id,
            created_at,
        })
    }
}

impl<'r> FromRow<'r, PgRow> for BankAccount
where
    Uuid: ::sqlx::decode::Decode<'r, Postgres>,
    Uuid: ::sqlx::types::Type<Postgres>,
{
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let balance_in_cents: i64 = row.try_get("balance")?;
        let balance: f64 = balance_in_cents as f64 / 100.0;
        let created_at: String = row.try_get("created_at")?;
        let user_id: Uuid = row.try_get("user_id")?;

        let raw_type: String = row.try_get("type")?;

        let account_type = AccountType::from_raw_string(raw_type.as_str())
            .map_err(|err| sqlx::Error::Decode(Box::new(err)))?;

        Ok(BankAccount {
            id,
            name,
            balance,
            account_type,
            user_id,
            created_at,
        })
    }
}
