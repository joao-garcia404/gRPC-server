use sqlx::postgres::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::proto::finance_control_server::FinanceControl;

use crate::models::bank_account;
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
            .map_err(|err| {
                println!("{err}");
                Status::invalid_argument("User not found".to_owned())
            })?;

        let account_type = bank_account::AccountType::from_raw_string(&input.account_type.as_str())
            .map_err(|err| Status::invalid_argument(err.get_message()))?;

        let account = bank_account::BankAccount::new(
            input.name,
            input.initial_balance * 100,
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
            account_id: account.id,
        };

        Ok(Response::new(response))
    }
}
