use chrono::Utc;
use uuid::Uuid;

#[derive(Debug)]
pub enum AccountType {
    CHECKING,
    INVESTMENT,
    CASH,
}

impl AccountType {
    pub fn from_raw_string(raw: &str) -> Result<AccountType, String> {
        match raw {
            "CHECKING" => Ok(AccountType::CHECKING),
            "INVESTMENT" => Ok(AccountType::INVESTMENT),
            "CASH" => Ok(AccountType::CASH),
            _ => Err("Invalid account type".to_owned()),
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
    ) -> Result<BankAccount, String> {
        let user_id = Uuid::try_parse(user_id.as_str())
            .map_err(|_err| "Failed to parse user_id as UUID".to_string())?
            .to_string();

        Ok(BankAccount {
            id: Uuid::new_v4().to_string(),
            name,
            balance,
            account_type,
            user_id: user_id,
            created_at: Utc::now().to_rfc3339(),
        })
    }
}
