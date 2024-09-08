use std::fmt;

use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum TransactionType {
    INCOME,
    OUTCOME,
}

impl TransactionType {
    pub fn from_proto(value: &i32) -> Result<Self, String> {
        match value {
            0 => Ok(TransactionType::INCOME),
            1 => Ok(TransactionType::OUTCOME),
            _ => Err("Invalid transaction type".to_owned()),
        }
    }
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::INCOME => write!(f, "INCOME"),
            TransactionType::OUTCOME => write!(f, "OUTCOME"),
        }
    }
}

#[derive(Debug)]
pub struct Transaction {
    pub id: String,
    pub amount: f64,
    pub transaction_type: TransactionType,
    pub origin_account_id: String,
    pub description: Option<String>,
    pub created_at: String,
}

impl Transaction {
    pub fn new(
        amount: f64,
        transaction_type: TransactionType,
        origin_account_id: String,
        description: Option<String>,
    ) -> Self {
        Transaction {
            id: Uuid::new_v4().to_string(),
            amount,
            description,
            origin_account_id,
            transaction_type,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}
