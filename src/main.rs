use dotenv::dotenv;
use proto::admin_server::{Admin, AdminServer};
use proto::finance_control_server::{FinanceControl, FinanceControlServer};
use sqlx::postgres::PgPool;
use std::env;
use std::{net::SocketAddr, sync::Arc};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

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

#[derive(Debug)]
struct Password {
    value: String,
}

impl Password {
    fn new(raw: String) -> Password {
        Password { value: raw }
    }

    fn get_hashed_value(&mut self) -> Result<(), String> {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();
        let hash_result = argon2.hash_password(self.value.as_bytes(), &salt);

        match hash_result {
            Ok(hashed) => {
                self.value = hashed.to_string();
                Ok(())
            }
            Err(_hash_error) => Err("Error at hashing password".to_owned()),
        }
    }
}

#[derive(Debug)]
struct User {
    id: String,
    name: String,
    email: String,
    password: Password,
    created_at: String,
}

impl User {
    fn new(name: String, email: String, raw_password: String) -> Result<User, String> {
        let mut user = User {
            id: Uuid::new_v4().to_string(),
            name,
            email,
            password: Password::new(raw_password),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        user.password.get_hashed_value()?;

        Ok(user)
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

        let _input = request.get_ref();

        let account_id = Uuid::new_v4().to_string();

        let response = proto::CreateBankAccountResponse { account_id };

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
