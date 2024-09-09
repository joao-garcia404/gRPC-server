use sqlx::postgres::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::proto::finance_control_server::FinanceControl;

use crate::models::bank_account;
use crate::models::transaction::{Transaction, TransactionType};
use crate::models::user::User;
use crate::proto;

pub struct FinanceControlService {
    pub state: Arc<tokio::sync::RwLock<u64>>,
    pub db_pool: PgPool,
}

impl FinanceControlService {
    async fn incremet_counter(&self) {
        let mut count = self.state.write().await;
        *count += 1;
        println!("Request count: {}", *count);
    }
}

#[tonic::async_trait]
impl FinanceControl for FinanceControlService {
    async fn register_user(
        &self,
        request: Request<proto::RegisterUserRequest>,
    ) -> Result<Response<proto::RegisterUserResponse>, Status> {
        self.incremet_counter().await;
        println!("Received a user registration request.");

        let input = request.into_inner();

        let user = User::new(input.name, input.email, input.password)
            .map_err(|_err| Status::unknown("Internal server error"))?;

        let query =
          "INSERT INTO users (id, name, email, password, created_at) VALUES ($1::uuid, $2, $3, $4, $5::timestamp)";

        sqlx::query(query)
            .bind(&user.id)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password.value)
            .bind(&user.created_at)
            .execute(&self.db_pool)
            .await
            .map_err(|err| {
                println!("Error while saving the user {:?}", err);
                Status::unknown("Internal server error")
            })?;

        let response = proto::RegisterUserResponse { user_id: user.id };

        Ok(Response::new(response))
    }

    async fn create_bank_account(
        &self,
        request: Request<proto::CreateBankAccountRequest>,
    ) -> Result<Response<proto::CreateBankAccountResponse>, Status> {
        self.incremet_counter().await;
        println!("Received a bank account creation request.");

        let input = request.into_inner();

        let user_exists_query = "SELECT * FROM users WHERE id::text = $1";

        let _ = sqlx::query(user_exists_query)
            .bind(&input.user_id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|_err| Status::invalid_argument("User not found".to_owned()))?;

        let account_type = bank_account::AccountType::from_raw_string(&input.account_type.as_str())
            .map_err(|err| Status::invalid_argument(err.get_message()))?;

        let account = bank_account::BankAccount::new(
            input.name,
            input.initial_balance * 100.0,
            account_type,
            input.user_id,
        )
        .map_err(|err| Status::invalid_argument(err.get_message()))?;

        let insert_bank_account_query =
      "INSERT INTO bank_accounts (id, name, balance, type, user_id, created_at) VALUES ($1::uuid, $2, $3, $4::bankaccounttype, $5::uuid, $6::timestamp)";

        sqlx::query(insert_bank_account_query)
            .bind(&account.id)
            .bind(&account.name)
            .bind(&account.balance)
            .bind(&account.account_type.to_string())
            .bind(&account.user_id)
            .bind(&account.created_at)
            .execute(&self.db_pool)
            .await
            .map_err(|err| {
                println!("Error while creating a bank account {:?}", err);
                Status::unknown("Internal server error")
            })?;

        let response = proto::CreateBankAccountResponse {
            account_id: account.id.to_string(),
        };

        Ok(Response::new(response))
    }

    async fn execute_transaction(
        &self,
        request: Request<proto::ExecuteTransactionRequest>,
    ) -> Result<Response<proto::ExecuteTransactionResponse>, Status> {
        self.incremet_counter().await;
        println!("Received a execute transaction request.");

        let input = request.into_inner();

        let mut account = sqlx::query(
            r#"SELECT id, name, balance, type, user_id, created_at::text
               FROM bank_accounts 
               WHERE id::text = $1"#,
        )
        .bind(&input.account_id)
        .map(|row| bank_account::BankAccount::from_pg_row(row))
        .fetch_one(&self.db_pool)
        .await
        .and_then(|result| {
            result.map_err(|err| {
                println!("Error: {:?}", err);

                sqlx::Error::RowNotFound
            })
        })
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => {
                Status::invalid_argument("Bank account not found".to_owned())
            }
            _ => Status::internal("Internal server error".to_owned()),
        })?;

        println!("Found account: {:?}", account);

        let transaction_type = TransactionType::from_proto(&input.transaction_type)
            .map_err(|err| Status::invalid_argument(err))?;

        if transaction_type == TransactionType::OUTCOME && account.balance < input.amount {
            return Err(Status::invalid_argument(
                "The account does not have enough funds for the transfer".to_owned(),
            ));
        }

        let transaction = Transaction::new(
            input.amount,
            transaction_type,
            input.account_id,
            input.description,
        );

        let _new_balance_result = account
            .update_balance(&transaction)
            .map_err(|err| Status::invalid_argument(err.get_message()))?;

        let mut txn = self.db_pool.begin().await.map_err(|err| {
            println!("Error while starting DB transaction: {:?}", err);
            Status::internal("Internal server error".to_owned())
        })?;

        let insert_transaction_query = r#"INSERT INTO transactions (id, amount, transaction_type, origin_account_id, description, created_at) VALUES ($1::uuid, $2, $3::transactiontype, $4::uuid, $5, $6::timestamp)"#;

        let amount_to_save = transaction.amount * 100.0;

        let _transaction_result = sqlx::query(insert_transaction_query)
            .bind(&transaction.id.to_string())
            .bind(&amount_to_save)
            .bind(&transaction.transaction_type.to_string())
            .bind(&transaction.origin_account_id.to_string())
            .bind(&transaction.description)
            .bind(&transaction.created_at)
            .execute(&mut *txn)
            .await
            .map_err(|err| {
                println!("Error while inserting transaction: {:?}", err);
                Status::internal("Internal server error".to_owned())
            })?;

        let update_account_balance_query = r#"
            UPDATE bank_accounts
            SET balance = $1
            WHERE id = $2::uuid
        "#;

        sqlx::query(update_account_balance_query)
            .bind(&account.balance)
            .bind(&account.id)
            .execute(&mut *txn)
            .await
            .map_err(|err| {
                println!("Error while updating account balance: {:?}", err);
                Status::internal("Internal server error".to_owned())
            })?;

        txn.commit().await.map_err(|err| {
            println!("Failed to commit insert transaction: {:?}", err);
            Status::internal("Internal server error".to_owned())
        })?;

        let response = proto::ExecuteTransactionResponse {
            transaction_id: transaction.id,
        };

        Ok(Response::new(response))
    }
}
