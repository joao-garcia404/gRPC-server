use proto::admin_server::{Admin, AdminServer};
use proto::finance_control_server::{FinanceControl, FinanceControlServer};
use std::{net::SocketAddr, sync::Arc};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

mod proto {
    tonic::include_proto!("finance_control");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("proto_descriptor");
}

type State = Arc<tokio::sync::RwLock<u64>>;

#[derive(Debug, Default)]
struct FinanceControlService {
    state: State,
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

        let _input = request.get_ref();

        let user_id = Uuid::new_v4().to_string();

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
    let addr: SocketAddr = "[::1]:50051".parse()?;

    let state = State::default();

    let finance = FinanceControlService {
        state: state.clone(),
    };

    let admin = AdminService {
        state: state.clone(),
    };

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .add_service(service)
        .add_service(AdminServer::new(admin))
        .add_service(FinanceControlServer::new(finance))
        .serve(addr)
        .await?;

    println!("Server running!");

    Ok(())
}
