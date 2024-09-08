use std::fmt;

use chrono::Utc;
use sqlx::{
    postgres::{PgRow, Postgres},
    FromRow, Row,
};
use uuid::Uuid;

#[derive(Debug)]
enum BankAccountErrorType {
    InvalidAccountType,
    UserIdParse,
}

#[derive(Debug)]
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

    pub fn get_message(&self) -> String {
        match self.action {
            BankAccountErrorType::InvalidAccountType => "Invalid account type".to_owned(),
            BankAccountErrorType::UserIdParse => "Failed to parse user id as a UUID".to_owned(),
        }
    }
}

impl fmt::Display for BankAccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.action {
            BankAccountErrorType::InvalidAccountType => write!(f, "Invalid account type"),
            BankAccountErrorType::UserIdParse => write!(f, "Failed to parse user id as a UUID"),
        }
    }
}

impl std::error::Error for BankAccountError {}

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
