use std::fmt;

use chrono::Utc;
use uuid::Uuid;

enum BankAccountErrorType {
    InvalidAccountType,
    UserIdParse,
}

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

#[derive(Debug)]
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
    pub id: String,
    pub name: String,
    pub balance: i64,
    pub account_type: AccountType,
    pub user_id: String,
    pub created_at: String,
}

impl BankAccount {
    pub fn new(
        name: String,
        balance: i64,
        account_type: AccountType,
        user_id: String,
    ) -> Result<BankAccount, BankAccountError> {
        let user_id = Uuid::try_parse(user_id.as_str())
            .map_err(|_err| BankAccountError::user_id_parse())?
            .to_string();

        Ok(BankAccount {
            id: Uuid::new_v4().to_string(),
            name,
            balance,
            account_type,
            user_id,
            created_at: Utc::now().to_rfc3339(),
        })
    }
}
