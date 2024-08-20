use dotenv::dotenv;
use proto::admin_server::{Admin, AdminServer};
use proto::finance_control_server::{FinanceControl, FinanceControlServer};
use sqlx::postgres::PgPool;
use std::env;
use std::{net::SocketAddr, sync::Arc};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use models::bank_account;
use models::user::User;

pub mod models;

mod proto {
    tonic::include_proto!("finance_control");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("proto_descriptor");
}

type State = Arc<tokio::sync::RwLock<u64>>;

#[derive(Debug)]
struct FinanceControlService {
    state: State,
    db_pool: PgPool,
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

        let user_id = Uuid::new_v4().to_string();

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

        let response = proto::RegisterUserResponse { user_id };

        Ok(Response::new(response))
    }

    async fn create_bank_account(
        &self,
        request: Request<proto::CreateBankAccountRequest>,
    ) -> Result<Response<proto::CreateBankAccountResponse>, Status> {
        self.incremet_counter().await;
        println!("Received a bank account creation request.");

        let input = request.into_inner();

        let account_type = bank_account::AccountType::from_raw_string(&input.account_type.as_str())
            .map_err(|err| Status::invalid_argument(err))?;

        let account = bank_account::BankAccount::new(
            input.name,
            input.initial_balance,
            account_type,
            input.user_id,
        )
        .map_err(|err| Status::invalid_argument(err))?;

        println!("{:?}", account);

        let response = proto::CreateBankAccountResponse {
            account_id: account.id,
        };

        Ok(Response::new(response))
    }
}

#[derive(Debug, Default)]
struct AdminService {
    state: State,
}

#[tonic::async_trait]
impl Admin for AdminService {
    async fn get_request_count(
        &self,
        _request: Request<proto::GetRequestCountRequest>,
    ) -> Result<Response<proto::GetRequestCountResponse>, Status> {
        let count = self.state.read().await;
        let response = proto::GetRequestCountResponse { count: *count };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_url: &str = &env::var("DATABASE_URL").unwrap();
    let pool = PgPool::connect(db_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let addr: SocketAddr = "[::1]:50051".parse()?;

    let state = State::default();

    let finance = FinanceControlService {
        state: state.clone(),
        db_pool: pool,
    };

    let admin = AdminService {
        state: state.clone(),
    };

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .add_service(reflection)
        .add_service(AdminServer::new(admin))
        .add_service(FinanceControlServer::new(finance))
        .serve(addr)
        .await?;

    println!("Server running!");

    Ok(())
}
